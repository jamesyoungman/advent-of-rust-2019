use std::fmt::{self, Display, Formatter};

/// Generic error type for when a typed error isn't useful.
#[derive(Debug)]
pub struct Fail(pub String);

impl Display for Fail {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl std::error::Error for Fail {}
