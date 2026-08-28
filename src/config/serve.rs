// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic)

use serde::{Deserialize, Serialize};

/// Serve-mode configuration.
///
/// An empty [`Serve::command`] uses the built-in static server; a non-empty
/// command is spawned as an external HTTP server (with `<output>` replaced by
/// the output directory).
#[derive(Deserialize, Debug, Serialize)]
pub struct Serve {
    pub edit: Option<String>,

    #[serde(default = "Serve::default_output")]
    pub output: String,

    #[serde(default)]
    pub command: Vec<String>,
}

impl Serve {
    fn default_output() -> String {
        "./.cache/publish".to_string()
    }
}

impl Default for Serve {
    fn default() -> Self {
        Self {
            edit: Some("vscode://file/".to_string()),
            output: Serve::default_output(),
            command: Vec::new(),
        }
    }
}
