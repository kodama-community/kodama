// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic)

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct Publish {
    pub rss: bool,
}
