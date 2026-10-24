//! For physics leaves whose result is in the flags and that write no memory
//! (`physics::gear_bit`, `physics::test_car_word`, `lower_free_gear`, ...): the hook's `dispatch` without a state
//! harness. Outside shadow mode the Rust version's registers and flags go to the game;
//! in shadow mode the original runs on a copy of the registers, the two are compared
//! (the registers and the flags named), and the original's are used.

/// What such a function leaves: eax (the value it had, where it is not changed), ecx and
/// ebx where they change, the carry and (when callers may read it) zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlagResult {
    pub eax: u32,
    pub ecx: Option<u32>,
    pub ebx: Option<u32>,
    pub carry: bool,
    pub zero: Option<bool>,
}

impl FlagResult {}
