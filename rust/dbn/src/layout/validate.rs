use crate::layout::StreamLayout;

/// Validates a [`StreamLayout`] before any field access, rejecting a malformed
/// table as a decode error.
///
/// POC stub: the spec §3.2 invariants are not yet implemented.
pub(crate) fn validate(_layout: &StreamLayout, _symbol_cstr_len: u16) -> crate::Result<()> {
    Ok(())
}
