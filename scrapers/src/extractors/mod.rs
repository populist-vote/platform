mod office;
mod party;
mod politician;

pub use office::*;
pub use party::*;
pub use politician::*;

#[inline]
fn owned_capture(capture: regex::Match) -> String {
    capture.as_str().to_string()
}

#[inline]
fn default_capture(captures: regex::Captures) -> Option<String> {
    captures.get(1).map(owned_capture)
}
