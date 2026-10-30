use std::sync::Arc;

/// A span of the original input. `start`/`end` are char offsets into `input`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub input: Arc<str>,
    pub start: usize,
    pub end: usize,
}

impl SourceLocation {
    pub fn new(input: impl Into<Arc<str>>, start: usize, end: usize) -> Self {
        SourceLocation {
            input: input.into(),
            start,
            end,
        }
    }

    pub fn range(start_loc: &SourceLocation, end_loc: &SourceLocation) -> Self {
        SourceLocation {
            input: start_loc.input.clone(),
            start: start_loc.start,
            end: end_loc.end,
        }
    }
}
