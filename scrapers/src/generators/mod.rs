pub mod election;
pub mod party;
pub mod politician;

pub mod co;


#[inline]
fn optional_state_str(state: Option<&db::State>) -> &str {
    state.map(|s| s.as_ref()).unwrap_or_default()
}
