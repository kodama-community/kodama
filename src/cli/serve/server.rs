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
    /// Bind a listener on `127.0.0.1` (incrementing the port until one is
    /// free) and start serving the current output directory in a background
    /// thread. The banner is printed later via [`BuiltinServer::banner`] so it
    /// appears after the watch setup messages.
    pub fn spawn() -> eyre::Result<Self> {
        let output_dir = environment::output_dir();
        let listener = bind_listener()?;
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
            let _ = thread.join();
        }
    }
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
    let _ = listener.set_nonblocking(true);
    while !stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let dir = output_dir.clone();
                std::thread::spawn(move || handle_connection(stream, dir.as_path()));
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
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
    use std::fs;

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
}
