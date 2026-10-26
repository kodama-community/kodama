// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use pulldown_cmark::CowStr;

/// split `url#:action` to `(url, action)`
pub fn url_action(dest_url: &CowStr<'_>) -> (String, String) {
    if let Some((base, action)) = dest_url.split_once("#:") {
        (base.to_string(), action.to_string())
    } else {
        (dest_url.to_string(), String::new())
    }
}

#[derive(Debug, PartialEq)]
pub enum State {
    /// Writable state
    None,

    /// Embeded contents
    Embed,

    /// Include contents
    Include,

    /// Shared for inline typst
    Shared,

    /// Export typst to HTML fragment
    Html,

    /// Inline typst
    InlineTypst,

    /// `display: inline`
    ImageSpan,

    /// `display: block; text-align: center`
    ImageBlock,

    /// `ImageBlock` with `<details>` code
    ImageCode,

    LocalLink,
    ExternalLink,
    AssetFile,
    UnsafeLink,
}

impl State {
    pub const fn strify(&self) -> &'static str {
        match self {
            State::None => "none",
            State::Embed => "embed",
            State::Include => "include",
            State::Shared => "shared",
            State::Html => "html",
            State::InlineTypst => "inline",
            State::ImageSpan => "span",
            State::ImageBlock => "block",
            State::ImageCode => "code",
            State::LocalLink => "local",       // style class name
            State::ExternalLink => "external", // style class name
            State::AssetFile => "asset",       // style class name
            State::UnsafeLink => "unsafe",     // style class name
        }
    }
}
