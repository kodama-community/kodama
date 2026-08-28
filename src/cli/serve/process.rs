// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use camino::Utf8Path;

use crate::environment;

use super::server::BuiltinServer;

/// The running HTTP server backing the `serve` command.
pub(super) enum ServeHandle {
    Builtin(BuiltinServer),
    External(std::process::Child),
}

impl ServeHandle {
    /// Stop the server: signal the built-in accept loop, or terminate and reap
    /// the external child process.
    pub fn kill(&mut self) {
        match self {
            ServeHandle::Builtin(server) => server.kill(),
            ServeHandle::External(child) => {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    /// The startup message to print after watch setup. External servers print
    /// their own banner through the relayed output.
    pub fn banner(&self) -> Option<String> {
        match self {
            ServeHandle::Builtin(server) => Some(server.banner()),
            ServeHandle::External(_) => None,
        }
    }
}

fn parse_command(command: &[String], output: &Utf8Path) -> std::process::Command {
    let mut serve = std::process::Command::new(&command[0]);
    for arg in &command[1..] {
        if arg == "<output>" {
            serve.arg(output);
            continue;
        }
        serve.arg(arg);
    }
    serve
}

fn relay_output<R: std::io::Read + Send + 'static>(stream: R, is_stderr: bool) {
    use std::io::{BufRead, BufReader};
    std::thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            match line {
                Ok(line) if is_stderr => color_print::ceprintln!("<r>[serve] Error: {line}</>"),
                Ok(line) => println!("[serve] {line}"),
                Err(err) => {
                    let label = if is_stderr { "stderr" } else { "stdout" };
                    color_print::ceprintln!("<r>[serve] {label} read error: {err}</>");
                    break;
                }
            }
        }
    });
}

fn spawn_external(command: &[String]) -> eyre::Result<std::process::Child> {
    let mut serve = parse_command(command, &environment::output_dir())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    if let Some(serve_stdout) = serve.stdout.take() {
        relay_output(serve_stdout, false);
    }

    if let Some(serve_stderr) = serve.stderr.take() {
        relay_output(serve_stderr, true);
    }

    Ok(serve)
}

pub(super) fn spawn_serve_process() -> eyre::Result<ServeHandle> {
    let command = environment::serve_command();
    if command.is_empty() {
        Ok(ServeHandle::Builtin(BuiltinServer::spawn()?))
    } else {
        Ok(ServeHandle::External(spawn_external(&command)?))
    }
}
