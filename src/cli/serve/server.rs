// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use eyre::eyre;

use crate::environment;

/// Default port the built-in server binds to, mirroring `miniserve`.
const DEFAULT_PORT: u16 = 8080;
/// Upper bound for a single incoming request (headers + body).
const MAX_REQUEST_SIZE: usize = 16 * 1024;

/// A minimal, dependency-free HTTP/1.1 static file server.
///
/// It serves the serve-mode output directory with the same semantics as the
/// default `miniserve <output> --index index.html --pretty-urls` invocation:
/// directory requests fall back to `index.html`, and extensionless paths (the
/// "pretty URLs" kodama emits when `build.pretty-urls` is enabled) fall back to
/// `<path>.html`.
pub struct BuiltinServer {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl BuiltinServer {
    /// Bind a listener on `127.0.0.1` and start serving the current output
    /// directory in a background thread. A `None` port scans upward from
    /// [`DEFAULT_PORT`] for a free port; `Some(port)` binds that port exactly,
    /// with `0` asking the operating system to choose one. The banner is
    /// printed later via [`BuiltinServer::banner`] so it appears after the
    /// watch setup messages.
    pub fn spawn(port: Option<u16>) -> eyre::Result<Self> {
        Self::spawn_with_listener(bind(port)?, environment::output_dir())
    }

    fn spawn_with_listener(listener: TcpListener, output_dir: Utf8PathBuf) -> eyre::Result<Self> {
        let addr = listener.local_addr()?;

        let stop = Arc::new(AtomicBool::new(false));
        let thread = std::thread::spawn({
            let stop = Arc::clone(&stop);
            move || accept_loop(listener, output_dir, stop)
        });

        Ok(Self {
            addr,
            stop,
            thread: Some(thread),
        })
    }

    /// The startup message announcing the served directory and address.
    pub fn banner(&self) -> String {
        color_print::cformat!(
            "<g>[serve] Serving `{}` at <b>http://127.0.0.1:{}</></>",
            environment::output_dir(),
            self.addr.port()
        )
    }

    /// Signal the accept loop to stop and wait for it to exit.
    pub fn kill(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            // Wake the blocking accept so shutdown does not wait for a request.
            let _ = TcpStream::connect(self.addr);
            let _ = thread.join();
        }
    }
}

fn bind(port: Option<u16>) -> eyre::Result<TcpListener> {
    match port {
        Some(port) => bind_exact(port),
        None => bind_listener(),
    }
}

/// Bind a user-requested port exactly, reporting the address on failure.
fn bind_exact(port: u16) -> eyre::Result<TcpListener> {
    TcpListener::bind(("127.0.0.1", port)).map_err(|err| {
        eyre!(
            "failed to bind 127.0.0.1:{port} for the built-in server (is the port already in use?): {err}"
        )
    })
}

fn bind_listener() -> eyre::Result<TcpListener> {
    for port in DEFAULT_PORT..=u16::MAX {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => return Ok(listener),
            Err(_) => continue,
        }
    }
    Err(eyre!(
        "no free port found between {DEFAULT_PORT} and {} on 127.0.0.1",
        u16::MAX
    ))
}

fn accept_loop(listener: TcpListener, output_dir: Utf8PathBuf, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                let dir = output_dir.clone();
                std::thread::spawn(move || handle_connection(stream, dir.as_path()));
            }
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
}

struct Request {
    method: String,
    /// Percent-decoded request path without the query string.
    path: String,
}

fn handle_connection(mut stream: TcpStream, output_dir: &Utf8Path) {
    let Some(data) = read_request(&mut stream) else {
        return;
    };
    let Some(request) = parse_request(&data) else {
        return;
    };
    let response = serve(output_dir, &request);
    write_response(&mut stream, &response);
}

fn read_request(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut data = Vec::with_capacity(1024);
    let mut buf = [0u8; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                data.extend_from_slice(&buf[..n]);
                if data.len() >= MAX_REQUEST_SIZE || has_header_end(&data) {
                    break;
                }
            }
            Err(_) => return None,
        }
    }
    Some(data)
}

fn has_header_end(data: &[u8]) -> bool {
    data.windows(4).any(|w| w == b"\r\n\r\n") || data.windows(2).any(|w| w == b"\n\n")
}

fn parse_request(data: &[u8]) -> Option<Request> {
    let text = std::str::from_utf8(data).ok()?;
    let request_line = text.lines().next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_ascii_uppercase();
    let target = parts.next()?;
    let path = target.split('?').next().unwrap_or(target);
    Some(Request {
        method,
        path: percent_decode(path),
    })
}

struct Response {
    status: &'static str,
    content_type: &'static str,
    content_length: usize,
    body: Vec<u8>,
}

fn serve(output_dir: &Utf8Path, request: &Request) -> Response {
    if !matches!(request.method.as_str(), "GET" | "HEAD") {
        return error_response("405 Method Not Allowed", "Method Not Allowed");
    }

    let Some(base) = resolve_path(output_dir, &request.path) else {
        return error_response("403 Forbidden", "Forbidden");
    };
    match resolve_file(&base) {
        Some(file) => serve_file(&file, &request.method),
        None => error_response("404 Not Found", "Not Found"),
    }
}

/// Map a resolved request path to a concrete file on disk, applying the
/// `--index` and `--pretty-urls` fallbacks.
fn resolve_file(base: &Utf8Path) -> Option<Utf8PathBuf> {
    if base.is_dir() {
        return Some(base.join("index.html"));
    }
    if base.is_file() {
        return Some(base.to_path_buf());
    }
    let name = base.file_name().unwrap_or_default();
    if !name.contains('.') {
        let pretty = Utf8PathBuf::from(format!("{}.html", base.as_str()));
        if pretty.is_file() {
            return Some(pretty);
        }
    }
    None
}

fn serve_file(path: &Utf8Path, method: &str) -> Response {
    let Ok(data) = std::fs::read(path) else {
        return error_response("404 Not Found", "Not Found");
    };
    Response {
        status: "200 OK",
        content_type: mime(path),
        content_length: data.len(),
        body: if method == "HEAD" { Vec::new() } else { data },
    }
}

fn error_response(status: &'static str, message: &str) -> Response {
    let body = format!("<!doctype html><html><body><h1>{message}</h1></body></html>").into_bytes();
    Response {
        status,
        content_type: "text/html; charset=utf-8",
        content_length: body.len(),
        body,
    }
}

/// Resolve a request path against the output directory, rejecting traversal.
fn resolve_path(output_dir: &Utf8Path, request_path: &str) -> Option<Utf8PathBuf> {
    if request_path.contains('\0') || request_path.contains('\\') {
        return None;
    }
    let mut joined = output_dir.to_path_buf();
    for component in request_path.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            return None;
        }
        joined.push(component);
    }
    Some(joined)
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = |c: u8| char::from(c).to_digit(16).map(|d| d as u8);
            if let (Some(hi), Some(lo)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push(hi * 16 + lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn write_response(stream: &mut TcpStream, response: &Response) {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status, response.content_type, response.content_length
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&response.body);
    let _ = stream.flush();
}

fn mime(path: &Utf8Path) -> &'static str {
    mime_from_ext(path.extension().unwrap_or_default())
}

fn mime_from_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        // The `kodama.reload` marker is polled as plain text by the live-reload script.
        "txt" | "reload" => "text/plain; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" | "oga" => "audio/ogg",
        "ogv" => "video/ogg",
        "wav" => "audio/wav",
        "md" => "text/markdown; charset=utf-8",
        "webmanifest" => "application/manifest+json",
        "yaml" | "yml" => "application/yaml; charset=utf-8",
        "toml" => "application/toml; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "tar" => "application/x-tar",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::mpsc, time::Duration};

    use super::*;

    fn serve_dir() -> Utf8PathBuf {
        let root = crate::test_io::case_dir("builtin-server");
        fs::create_dir_all(root.as_std_path()).unwrap();
        root
    }

    fn request(method: &str, target: &str) -> Request {
        let target = target.split('?').next().unwrap_or(target);
        Request {
            method: method.to_string(),
            path: percent_decode(target),
        }
    }

    fn write(root: &Utf8Path, relative: &str, content: &[u8]) {
        let file = root.join(relative);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, content).unwrap();
    }

    fn start_server(root: &Utf8Path) -> BuiltinServer {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        BuiltinServer::spawn_with_listener(listener, root.to_path_buf()).unwrap()
    }

    fn tcp_request(addr: SocketAddr, method: &str, target: &str) -> String {
        let timeout = Duration::from_secs(2);
        let mut stream = TcpStream::connect_timeout(&addr, timeout).unwrap();
        stream.set_read_timeout(Some(timeout)).unwrap();
        stream.set_write_timeout(Some(timeout)).unwrap();
        write!(
            stream,
            "{method} {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    #[test]
    fn test_server_handles_sequential_tcp_requests() {
        let root = serve_dir();
        write(&root, "index.html", b"<h1>home</h1>");
        let mut server = start_server(&root);

        for method in ["GET", "HEAD", "GET"] {
            let response = tcp_request(server.addr, method, "/");
            let (headers, body) = response.split_once("\r\n\r\n").unwrap();
            assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"));
            assert!(headers.contains("Content-Length: 13\r\n"));
            assert_eq!(
                body,
                if method == "HEAD" {
                    ""
                } else {
                    "<h1>home</h1>"
                }
            );
        }

        server.kill();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_server_stops_before_and_after_serving_requests() {
        let root = serve_dir();
        write(&root, "index.html", b"home");

        for warm_up in [false, true] {
            let mut server = start_server(&root);
            if warm_up {
                assert!(tcp_request(server.addr, "GET", "/").ends_with("home"));
            }
            let (stopped, receiver) = mpsc::channel();
            let shutdown = std::thread::spawn(move || {
                server.kill();
                server.kill();
                assert!(server.thread.is_none());
                stopped.send(()).unwrap();
            });
            receiver
                .recv_timeout(Duration::from_secs(2))
                .expect("server did not stop while waiting for a connection");
            shutdown.join().unwrap();
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_serves_exact_file() {
        let root = serve_dir();
        write(&root, "index.html", b"<h1>home</h1>");

        let response = serve(&root, &request("GET", "/index.html"));
        assert_eq!(response.status, "200 OK");
        assert_eq!(response.body, b"<h1>home</h1>");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_serves_directory_index() {
        let root = serve_dir();
        write(&root, "notes/index.html", b"<h1>notes</h1>");

        let response = serve(&root, &request("GET", "/notes/"));
        assert_eq!(response.status, "200 OK");
        assert_eq!(response.body, b"<h1>notes</h1>");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_pretty_url_falls_back_to_html() {
        let root = serve_dir();
        write(&root, "notes/a.html", b"<h1>a</h1>");

        let response = serve(&root, &request("GET", "/notes/a"));
        assert_eq!(response.status, "200 OK");
        assert_eq!(response.body, b"<h1>a</h1>");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_file_with_extension_is_served_directly() {
        let root = serve_dir();
        write(&root, "kodama.reload", b"7");

        let response = serve(&root, &request("GET", "/kodama.reload"));
        assert_eq!(response.status, "200 OK");
        assert_eq!(response.body, b"7");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_percent_encoded_path() {
        let root = serve_dir();
        write(&root, "notes/hello world.html", b"<h1>hi</h1>");

        let response = serve(&root, &request("GET", "/notes/hello%20world"));
        assert_eq!(response.status, "200 OK");
        assert_eq!(response.body, b"<h1>hi</h1>");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_query_string_is_ignored() {
        let root = serve_dir();
        write(&root, "index.html", b"<h1>home</h1>");

        let response = serve(&root, &request("GET", "/index.html?t=1"));
        assert_eq!(response.status, "200 OK");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_head_reports_length_without_body() {
        let root = serve_dir();
        write(&root, "index.html", b"body-bytes");

        let response = serve(&root, &request("HEAD", "/index.html"));
        assert_eq!(response.status, "200 OK");
        assert_eq!(response.content_length, "body-bytes".len());
        assert!(response.body.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_missing_file_is_404() {
        let root = serve_dir();
        let response = serve(&root, &request("GET", "/nope.html"));
        assert_eq!(response.status, "404 Not Found");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_path_traversal_is_forbidden() {
        let root = serve_dir();
        let response = serve(&root, &request("GET", "/../Cargo.toml"));
        assert_eq!(response.status, "403 Forbidden");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_non_get_head_method_is_405() {
        let root = serve_dir();
        let response = serve(&root, &request("POST", "/index.html"));
        assert_eq!(response.status, "405 Method Not Allowed");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_mime_types() {
        let root = serve_dir();
        let files = [
            ("page.html", "text/html"),
            ("style.css", "text/css"),
            ("app.js", "application/javascript"),
            ("data.json", "application/json"),
            ("img.png", "image/png"),
            ("img.svg", "image/svg+xml"),
            ("marker.reload", "text/plain"),
        ];
        for (name, expected) in files {
            write(&root, name, b"x");
            let response = serve(&root, &request("GET", name));
            assert_eq!(response.status, "200 OK", "unexpected status for {name}");
            assert!(
                response.content_type.starts_with(expected),
                "{name}: expected {expected}, got {}",
                response.content_type
            );
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_fixed_port_already_in_use_is_reported() {
        let occupied = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = occupied.local_addr().unwrap().port();

        let err = bind(Some(port)).unwrap_err();
        assert!(err.to_string().contains(&format!("127.0.0.1:{port}")));
    }

    #[test]
    fn test_port_zero_asks_the_os_for_a_port() {
        let listener = bind(Some(0)).unwrap();
        assert_ne!(listener.local_addr().unwrap().port(), 0);
    }
}
