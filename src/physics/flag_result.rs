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
