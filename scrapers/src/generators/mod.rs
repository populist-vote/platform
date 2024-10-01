mod election;
mod office;
mod party;
mod race;

pub use election::*;
pub use office::*;
pub use party::*;
pub use race::*;

#[inline]
fn optional_state_str(state: Option<&db::State>) -> &str {
    state.map(|s| s.as_ref()).unwrap_or_default()
}
