//! For physics leaves whose result is in the flags and that write no memory
//! (`physics::gear_bit`, `physics::test_car_word`, `lower_free_gear`, ...): the hook's `dispatch` without a state
//! harness. Outside shadow mode the Rust version's registers and flags go to the game;
//! in shadow mode the original runs on a copy of the registers, the two are compared
//! (the registers and the flags named), and the original's are used.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, shadow};
use crate::screens::harness::CallOriginal;

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

impl FlagResult {
    pub fn apply(&self, r: &mut Regs) {
        r.eax = self.eax;
        if let Some(v) = self.ecx {
            r.ecx = v;
        }
        if let Some(v) = self.ebx {
            r.ebx = v;
        }
        r.set_carry(self.carry);
        if let Some(z) = self.zero {
            r.set_zero(z);
        }
    }

    /// True if `after` (the original's) is this, every other register kept from `before`.
    pub fn matches(&self, before: &Regs, after: &Regs) -> bool {
        use crate::framework::{EAX, EBX, ECX};
        let mask = EAX | self.ecx.map_or(0, |_| ECX) | self.ebx.map_or(0, |_| EBX);
        after.eax == self.eax
            && self.ecx.is_none_or(|v| after.ecx == v)
            && self.ebx.is_none_or(|v| after.ebx == v)
            && after.carry() == self.carry
            && self
                .zero
                .is_none_or(|z| crate::audio::buffer_play::zero(after) == z)
            && after.preserved_except(before, mask)
    }
}

pub fn dispatch_flags(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: &dyn Fn(&Regs) -> Result<FlagResult, Refused>,
) {
    hook.count_call();
    let mine = match rust(r) {
        Ok(m) => m,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
            return;
        }
    };
    if !shadow() {
        mine.apply(r);
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if !mine.matches(r, &o) {
        crate::screens::harness::mismatch_unless_disturbed(hook, || {
            format!(
                "ecx {:#x}: original eax {:#x} carry {} zero {}; rust {mine:?}",
                r.ecx,
                o.eax,
                o.carry(),
                crate::audio::buffer_play::zero(&o)
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}
