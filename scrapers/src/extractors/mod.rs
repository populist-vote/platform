pub mod party;
pub mod politician;

pub mod co;

#[inline]
fn owned_capture(capture: regex::Match) -> String {
    capture.as_str().to_string()
}

#[inline]
fn default_capture(captures: regex::Captures) -> Option<String> {
    captures.get(1).map(owned_capture)
}
