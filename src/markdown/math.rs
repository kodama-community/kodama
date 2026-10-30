// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic)

use tonic::{HtmlRenderConfig, Settings, TrustPolicy};

use crate::{environment, slug::Slug};

fn delimited(source: &str, display: bool) -> String {
    let delimiter = if display { "$$" } else { "$" };
    format!("{delimiter}{source}{delimiter}")
}

/// Serialize math at a rich-text output boundary. With pre-rendering disabled,
/// emit escaped, delimited source for browser rendering. Otherwise compile with
/// independent macro state and show errors in place without a source fallback.
/// Plain-text consumers must keep the original events and use `write_leaf`.
pub fn render_html(source: &str, display: bool, slug: Slug) -> String {
    if !environment::pre_render_math() {
        return htmlize::escape_text(delimited(source, display)).into_owned();
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
            color_print::ceprintln!("<r>Math error in `{}`: {}</>", slug, error);
            let message = htmlize::escape_text(format!("Math error: {error}"));
            let display_class = if display { " katex-display" } else { "" };
            format!("<span class=\"katex-error{display_class}\">{message}</span>")
        }
    }
}
