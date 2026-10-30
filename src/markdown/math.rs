// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic)

use tonic::{HtmlRenderConfig, Settings, TrustPolicy};

use crate::{
    compiler::{Diagnostic, ParseDiagnostics},
    environment,
    slug::Slug,
};

fn delimited(source: &str, display: bool) -> String {
    let delimiter = if display { "$$" } else { "$" };
    format!("{delimiter}{source}{delimiter}")
}

/// Serialize math at a rich-text output boundary. With pre-rendering disabled,
/// wrap escaped, delimited source in a math span for browser rendering. Otherwise
/// compile with independent macro state and show errors without a source fallback.
/// Plain-text consumers must keep the original events and use `write_leaf`.
pub fn render_html(source: &str, display: bool, slug: Slug, diagnostics: &ParseDiagnostics) -> String {
    if !environment::pre_render_math() {
        let mode = if display { "display" } else { "inline" };
        let source = htmlize::escape_text(delimited(source, display));
        return format!(r#"<span class="math math-{mode}">{source}</span>"#);
    }

    let mut settings = Settings {
        display_mode: display,
        // Match the existing browser renderer instead of adding a second policy.
        trust: TrustPolicy::Trusted,
        ..Settings::new()
    };
    let config = HtmlRenderConfig {
        display_mode: display,
        min_rule_thickness: 0.05,
        ..HtmlRenderConfig::new()
    };
    let rendered =
        tonic::parse(source, &mut settings).and_then(|nodes| tonic::render_html(&nodes, config));
    match rendered {
        Ok(html) => html,
        Err(error) => {
            diagnostics.borrow_mut().push(Diagnostic::error(format!("{error} at {slug}")));
            htmlize::escape_text(error.to_string()).into_owned()
        }
    }
}
