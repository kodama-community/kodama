mod atomic;
mod block;
mod config;
#[allow(clippy::module_inception)]
mod unicode;
mod unicode_array;
mod unicode_cd;
mod unicode_frac;
mod unicode_width;

pub use block::Block;
pub use config::{LineStyle, RenderConfig, line_style_frac_bar, unicode_text_spacing};
pub use unicode::render;
