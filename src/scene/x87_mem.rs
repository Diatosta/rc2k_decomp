//! Memory operands of the blocks that were x87 assembly, as plain functions: a base pointer
//! and a byte offset, unaligned, in the `X` model of `math/x87_float.rs`.
//!
//! `f` is `fld dword`, `i` is `fild dword`, `t80` is `fld tbyte`; `wf` is `fstp dword`,
//! `wi` is `fistp dword`, `wt80` is `fstp tbyte`, `wh` stores a status word.

#![allow(unsafe_code)]

use crate::math::x87_float::{X, f80, from_f80, int, ld, st, to_int};

/// An operand's base: a block the caller holds, reached by byte offset.
pub(crate) trait Base: Copy {
    /// The dword at `off`.
    #[track_caller]
    fn dword(self, off: usize) -> u32;
    /// `value` as the dword at `off`.
    #[track_caller]
    fn set_dword(self, off: usize, value: u32);
    /// `value` as the 16 bits at `off`.
    #[track_caller]
    fn set_word(self, off: usize, value: u16);
    /// The ten bytes at `off`.
    #[track_caller]
    fn tbyte(self, off: usize) -> [u8; 10];
    /// `value` as the ten bytes at `off`.
    #[track_caller]
    fn set_tbyte(self, off: usize, value: [u8; 10]);
}

/// The ten bytes at `off` of a base whose dwords are `base.dword`'s.
#[track_caller]
fn tbyte_of_dwords(base: impl Base, off: usize) -> [u8; 10] {
    std::array::from_fn(|k| base.dword((off + k) & !3).to_le_bytes()[(off + k) % 4])
}

/// `value` as the ten bytes at `off` of a base whose dwords are `base`'s.
#[track_caller]
fn set_tbyte_of_dwords(base: impl Base, off: usize, value: [u8; 10]) {
    for (k, byte) in value.into_iter().enumerate() {
        let at = (off + k) & !3;
        let mut bytes = base.dword(at).to_le_bytes();
        bytes[(off + k) % 4] = byte;
        base.set_dword(at, u32::from_le_bytes(bytes));
    }
}

/// `value` as the low 16 bits of the dword at `off`, the high 16 kept.
#[track_caller]
fn set_word_of_dword(base: impl Base, off: usize, value: u16) {
    base.set_dword(off, base.dword(off) & 0xFFFF_0000 | u32::from(value));
}

/// A block of dwords the caller holds: each operand a bounds check on the block, nothing more.
/// What the arithmetic of the hot paths works on (a game address costs a look through the
/// tables, a pointer a look at the thread's stack, for every operand).
impl Base for &[std::cell::Cell<u32>] {
    #[inline(always)]
    fn dword(self, off: usize) -> u32 {
        assert!(off.is_multiple_of(4));
        self[off / 4].get()
    }
    #[inline(always)]
    fn set_dword(self, off: usize, value: u32) {
        assert!(off.is_multiple_of(4));
        self[off / 4].set(value);
    }
    #[inline(always)]
    fn set_word(self, off: usize, value: u16) {
        assert!(off.is_multiple_of(4));
        let cell = &self[off / 4];
        cell.set(cell.get() & 0xFFFF_0000 | u32::from(value));
    }
    fn tbyte(self, off: usize) -> [u8; 10] {
        std::array::from_fn(|k| self[(off + k) / 4].get().to_le_bytes()[(off + k) % 4])
    }
    fn set_tbyte(self, off: usize, value: [u8; 10]) {
        for (k, byte) in value.into_iter().enumerate() {
            let cell = &self[(off + k) / 4];
            let mut bytes = cell.get().to_le_bytes();
            bytes[(off + k) % 4] = byte;
            cell.set(u32::from_le_bytes(bytes));
        }
    }
}

/// A block of dwords only read.
impl Base for &[u32] {
    #[inline(always)]
    fn dword(self, off: usize) -> u32 {
        assert!(off.is_multiple_of(4));
        self[off / 4]
    }
    fn set_dword(self, _off: usize, _value: u32) {
        unreachable!("a store to a block only read");
    }
    fn set_word(self, _off: usize, _value: u16) {
        unreachable!("a store to a block only read");
    }
    fn tbyte(self, off: usize) -> [u8; 10] {
        std::array::from_fn(|k| self[(off + k) / 4].to_le_bytes()[(off + k) % 4])
    }
    fn set_tbyte(self, _off: usize, _value: [u8; 10]) {
        unreachable!("a store to a block only read");
    }
}

/// A game variable as the one dword of an operand.
impl Base for &crate::session::Dword {
    #[inline(always)]
    fn dword(self, off: usize) -> u32 {
        assert!(off == 0);
        self.get()
    }
    #[inline(always)]
    fn set_dword(self, off: usize, value: u32) {
        assert!(off == 0);
        self.set(value);
    }
    fn set_word(self, off: usize, value: u16) {
        set_word_of_dword(self, off, value);
    }
    fn tbyte(self, off: usize) -> [u8; 10] {
        tbyte_of_dwords(self, off)
    }
    fn set_tbyte(self, off: usize, value: [u8; 10]) {
        set_tbyte_of_dwords(self, off, value);
    }
}

/// A polygon vertex (`TLVertex`): its eight dwords by their offsets, 0 .. 0x1C.
impl Base for &crate::d3d::vertices_store::Vertex {
    #[inline(always)]
    fn dword(self, off: usize) -> u32 {
        match off {
            0x0 => self.x.bits(),
            0x4 => self.y.bits(),
            0x8 => self.z.bits(),
            0xC => self.rhw.bits(),
            0x10 => self.colour.get(),
            0x14 => self.specular.get(),
            0x18 => self.u.bits(),
            0x1C => self.v.bits(),
            _ => panic!("a vertex has no dword at {off:#x}"),
        }
    }
    #[inline(always)]
    fn set_dword(self, off: usize, value: u32) {
        match off {
            0x0 => self.x.set_bits(value),
            0x4 => self.y.set_bits(value),
            0x8 => self.z.set_bits(value),
            0xC => self.rhw.set_bits(value),
            0x10 => self.colour.set(value),
            0x14 => self.specular.set(value),
            0x18 => self.u.set_bits(value),
            0x1C => self.v.set_bits(value),
            _ => panic!("a vertex has no dword at {off:#x}"),
        }
    }
    fn set_word(self, off: usize, value: u16) {
        set_word_of_dword(self, off, value);
    }
    fn tbyte(self, off: usize) -> [u8; 10] {
        tbyte_of_dwords(self, off)
    }
    fn set_tbyte(self, off: usize, value: [u8; 10]) {
        set_tbyte_of_dwords(self, off, value);
    }
}

/// Ten-byte values only read, the operands of `fld tbyte`.
impl Base for &[[u8; 10]] {
    fn dword(self, _off: usize) -> u32 {
        unreachable!("a dword of ten-byte values")
    }
    fn set_dword(self, _off: usize, _value: u32) {
        unreachable!("a store to values only read");
    }
    fn set_word(self, _off: usize, _value: u16) {
        unreachable!("a store to values only read");
    }
    fn set_tbyte(self, _off: usize, _value: [u8; 10]) {
        unreachable!("a store to values only read");
    }
    #[inline(always)]
    fn tbyte(self, off: usize) -> [u8; 10] {
        assert!(off.is_multiple_of(10));
        self[off / 10]
    }
}

/// Ten-byte values written, the results of `fstp tbyte`.
impl Base for &[std::cell::Cell<[u8; 10]>] {
    fn dword(self, _off: usize) -> u32 {
        unreachable!("a dword of ten-byte values")
    }
    fn set_dword(self, _off: usize, _value: u32) {
        unreachable!("a dword of ten-byte values");
    }
    fn set_word(self, _off: usize, _value: u16) {
        unreachable!("a word of ten-byte values");
    }
    fn tbyte(self, off: usize) -> [u8; 10] {
        assert!(off.is_multiple_of(10));
        self[off / 10].get()
    }
    #[inline(always)]
    fn set_tbyte(self, off: usize, value: [u8; 10]) {
        assert!(off.is_multiple_of(10));
        self[off / 10].set(value);
    }
}

/// `block` as cells, for the arithmetic that writes it.
pub(crate) fn cells(block: &mut [u32]) -> &[std::cell::Cell<u32>] {
    std::cell::Cell::from_mut(block).as_slice_of_cells()
}

/// A test's operand buffer by game address (the assembly references' operands), read and
/// written through the checked region lookup.
#[cfg(test)]
mod by_address {
    use super::Base;

    /// The `T` at the operand, read through the checked place layer (the program ends outside
    /// one live region, `pointers::read_or_end`).
    #[track_caller]
    pub(super) fn load<T: crate::pointers::Plain>(address: usize, off: usize) -> T {
        crate::pointers::read_or_end(address.wrapping_add(off) as u32, "an x87 operand")
    }

    /// `value` written at the operand (as `load`).
    #[track_caller]
    pub(super) fn store<T: crate::pointers::Plain>(address: usize, off: usize, value: T) {
        crate::pointers::write_or_end(address.wrapping_add(off) as u32, value, "an x87 operand");
    }

    /// A base that is a game address.
    macro_rules! base_at {
        ($t:ty, $address:expr) => {
            impl Base for $t {
                #[track_caller]
                fn dword(self, off: usize) -> u32 {
                    load(($address)(self), off)
                }
                #[track_caller]
                fn set_dword(self, off: usize, value: u32) {
                    store(($address)(self), off, value);
                }
                #[track_caller]
                fn set_word(self, off: usize, value: u16) {
                    store(($address)(self), off, value);
                }
                #[track_caller]
                fn tbyte(self, off: usize) -> [u8; 10] {
                    load(($address)(self), off)
                }
                #[track_caller]
                fn set_tbyte(self, off: usize, value: [u8; 10]) {
                    store(($address)(self), off, value);
                }
            }
        };
    }

    base_at!(usize, |a: usize| a);
    #[cfg(target_arch = "x86")]
    base_at!(*mut u8, |p: *mut u8| crate::pointers::game_address(p)
        as usize);
    #[cfg(target_arch = "x86")]
    base_at!(*const u8, |p: *const u8| crate::pointers::game_address(p)
        as usize);
}

/// A place in the game's memory, read and written within its own bounds.
#[cfg(test)]
impl Base for crate::pointers::GamePtr {
    #[track_caller]
    fn dword(self, off: usize) -> u32 {
        bounded(self.offset(off as u32).read("an x87 operand"))
    }
    #[track_caller]
    fn set_dword(self, off: usize, value: u32) {
        bounded(self.offset(off as u32).write(value, "an x87 operand"));
    }
    #[track_caller]
    fn set_word(self, off: usize, value: u16) {
        bounded(self.offset(off as u32).write(value, "an x87 operand"));
    }
    #[track_caller]
    fn tbyte(self, off: usize) -> [u8; 10] {
        bounded(self.offset(off as u32).read("an x87 operand"))
    }
    #[track_caller]
    fn set_tbyte(self, off: usize, value: [u8; 10]) {
        bounded(self.offset(off as u32).write(value, "an x87 operand"));
    }
}

/// A checked access's value, or the program ended naming it.
#[cfg(test)]
#[track_caller]
fn bounded<T>(done: Result<T, crate::errors::BadInput>) -> T {
    done.unwrap_or_else(|e| crate::fatal::raise::end("an x87 operand", e))
}

/// `fld dword ptr [base + off]`.
#[track_caller]
pub(crate) fn fld(base: impl Base, off: usize) -> X {
    ld(base.dword(off))
}

/// A test buffer's dword as a cell (checked in one live region, as `fld`).
#[cfg(test)]
pub(crate) struct HostCell(pub usize);

#[cfg(test)]
impl crate::session::Cell32 for HostCell {
    fn bits(&self) -> u32 {
        by_address::load(self.0, 0)
    }
    fn set_bits(&self, bits: u32) {
        by_address::store(self.0, 0, bits);
    }
}

/// The `T` at `off` bytes into a test buffer at `base` (the plain versions' typed operands).
#[cfg(test)]
pub(crate) fn view<T>(base: usize, off: usize) -> &'static T {
    // SAFETY: the buffer is lent for the run (`block_vectors`), aligned for its cells.
    unsafe { &*(crate::pointers::host_at(base + off) as *const T) }
}

/// `fild dword ptr [base + off]`.
#[track_caller]
pub(crate) fn fild(base: impl Base, off: usize) -> X {
    int(base.dword(off) as i32)
}

/// `fld tbyte ptr [base + off]`.
#[track_caller]
pub(crate) fn fld80(base: impl Base, off: usize) -> X {
    from_f80(base.tbyte(off))
}

/// `fstp dword ptr [base + off]`: the value rounded to a float as the control word says.
#[track_caller]
pub(crate) fn fstp(base: impl Base, off: usize, x: X) {
    base.set_dword(off, st(x));
}

/// `fistp dword ptr [base + off]`: the value rounded to a whole number as the control word
/// says.
#[track_caller]
pub(crate) fn fistp(base: impl Base, off: usize, x: X) {
    base.set_dword(off, to_int(x) as u32);
}

/// `fstp tbyte ptr [base + off]`.
#[track_caller]
pub(crate) fn fstp80(base: impl Base, off: usize, x: X) {
    base.set_tbyte(off, f80(x));
}

/// `fnstsw word ptr [base + off]`.
#[track_caller]
pub(crate) fn fnstsw(base: impl Base, off: usize, status: u16) {
    base.set_word(off, status);
}

/// `mov dword ptr [base + off], value`.
#[track_caller]
pub(crate) fn mov(base: impl Base, off: usize, value: u32) {
    base.set_dword(off, value);
}

#[cfg(test)]
mod block_tests {
    use super::*;

    /// Each store and load on a block of cells does what it does through a game address.
    #[test]
    fn a_block_of_cells_behaves_as_memory() {
        let mut by_pointer = [0x4040_0000u32, 7, 0xAAAA_BBBB, 0];
        let mut by_cells = by_pointer;
        // The test's own block on its stack, by game address (`usize`, checked).
        let p = crate::pointers::game_address(by_pointer.as_mut_ptr()) as usize;
        let c = cells(&mut by_cells);
        assert_eq!(fld(p, 0).0, fld(c, 0).0);
        assert_eq!(fild(p, 4).0, fild(c, 4).0);
        fstp(p, 0xC, X(1.5));
        fstp(c, 0xC, X(1.5));
        fistp(p, 4, X(-2.0));
        fistp(c, 4, X(-2.0));
        fnstsw(p, 8, 0x1234);
        fnstsw(c, 8, 0x1234);
        mov(p, 0, 9);
        mov(c, 0, 9);
        let in_cells: Vec<u32> = c.iter().map(std::cell::Cell::get).collect();
        assert_eq!(by_pointer[..], in_cells[..]);
        assert_eq!(by_pointer[2], 0xAAAA_1234);
    }
}
