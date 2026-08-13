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
