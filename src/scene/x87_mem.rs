//! Memory operands of the blocks that were x87 assembly, as plain functions: a base pointer
//! and a byte offset, unaligned, in the `X` model of `math/x87_float.rs`.
//!
//! `f` is `fld dword`, `i` is `fild dword`, `t80` is `fld tbyte`; `wf` is `fstp dword`,
//! `wi` is `fistp dword`, `wt80` is `fstp tbyte`, `wh` stores a status word.

use crate::math::x87_float::{X, f80, from_f80, int, ld, st, to_int};

/// An operand's base: an address, as the runs hold it, or a pointer.
pub(crate) trait Base: Copy {
    fn address(self) -> usize;
    /// The host pointer itself, when the base is one.
    fn direct(self) -> Option<*mut u8> {
        None
    }
}

impl Base for usize {
    fn address(self) -> usize {
        self
    }
}

impl Base for *const u8 {
    fn address(self) -> usize {
        crate::pointers::game_address(self) as usize
    }
    fn direct(self) -> Option<*mut u8> {
        Some(self.cast_mut())
    }
}

impl Base for *mut u8 {
    fn address(self) -> usize {
        crate::pointers::game_address(self) as usize
    }
    fn direct(self) -> Option<*mut u8> {
        Some(self)
    }
}

/// The operand's address, its `len` bytes checked to lie in one region (the program ends
/// otherwise, `pointers::check_or_end`).
#[track_caller]
#[inline(always)]
fn operand(base: impl Base, off: usize, len: usize) -> *mut u8 {
    // A pointer to one of the program's statics or to this thread's stack needs no trip through
    // the address tables (these run many times a physics tick); the same bounds hold it.
    if let Some(p) = base.direct() {
        let at = p.wrapping_add(off);
        if crate::address_map::in_image(at as usize, len)
            || crate::os::in_thread_stack(at as usize, len)
        {
            return at;
        }
    }
    // A game address (the race core's base and an RVA, a record's address).
    let at = base.address().wrapping_add(off) as u32;
    crate::pointers::check_or_end(at, len, "an x87 operand");
    crate::pointers::host(at)
}

/// The `T` at the operand.
#[track_caller]
#[inline(always)]
fn load<T: Copy>(base: impl Base, off: usize) -> T {
    // SAFETY: checked in one live region; any bits make a value of the plain types used here.
    unsafe {
        operand(base, off, size_of::<T>())
            .cast::<T>()
            .read_unaligned()
    }
}

/// `value` written at the operand.
#[track_caller]
#[inline(always)]
fn store<T: Copy>(base: impl Base, off: usize, value: T) {
    // SAFETY: checked in one live region.
    unsafe {
        operand(base, off, size_of::<T>())
            .cast::<T>()
            .write_unaligned(value)
    }
}

/// `fld dword ptr [base + off]`.
#[track_caller]
pub(crate) fn fld(base: impl Base, off: usize) -> X {
    ld(load(base, off))
}

/// A test buffer's dword as a cell (checked in one live region, as `fld`).
#[cfg(all(test, target_arch = "x86"))]
pub(crate) struct HostCell(pub usize);

#[cfg(all(test, target_arch = "x86"))]
impl crate::session::Cell32 for HostCell {
    fn bits(&self) -> u32 {
        load(self.0, 0)
    }
    fn set_bits(&self, bits: u32) {
        store(self.0, 0, bits)
    }
}

/// `fild dword ptr [base + off]`.
#[track_caller]
pub(crate) fn fild(base: impl Base, off: usize) -> X {
    int(load(base, off))
}

/// `fld tbyte ptr [base + off]`.
#[track_caller]
pub(crate) fn fld80(base: impl Base, off: usize) -> X {
    from_f80(load(base, off))
}

/// `fstp dword ptr [base + off]`: the value rounded to a float as the control word says.
#[track_caller]
pub(crate) fn fstp(base: impl Base, off: usize, x: X) {
    store(base, off, st(x))
}

/// `fistp dword ptr [base + off]`: the value rounded to a whole number as the control word
/// says.
#[track_caller]
pub(crate) fn fistp(base: impl Base, off: usize, x: X) {
    store(base, off, to_int(x))
}

/// `fstp tbyte ptr [base + off]`.
#[track_caller]
pub(crate) fn fstp80(base: impl Base, off: usize, x: X) {
    store(base, off, f80(x))
}

/// `fnstsw word ptr [base + off]`.
#[track_caller]
pub(crate) fn fnstsw(base: impl Base, off: usize, status: u16) {
    store(base, off, status)
}

/// `mov dword ptr [base + off], value`.
#[track_caller]
pub(crate) fn mov(base: impl Base, off: usize, value: u32) {
    store(base, off, value)
}
