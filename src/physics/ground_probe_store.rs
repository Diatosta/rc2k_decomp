//! The ground probe's marks (IDA 0x620458 .. 0x620474; phase 7, typed): the edge test's
//! product (a qword) and the mark that tells the ground search how it was called.
//! `#[repr(C)]` with the image's layout, over the race block's bytes (`GROUND_PROBE`).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct GroundProbe {
    /// +0x0 (0x620458): `dword_620458`: not used by the Rust side.
    pub unknown_0: Dword,
    /// +0x4 (0x62045C): `dword_62045C`: not used by the Rust side.
    pub unknown_4: Dword,
    /// +0x8 (0x620460): `qword_620460`: the product of the edge test, low then high dword (`physics::crossed_edge_contact`, `physics::ground`, `race::car_shadow`).
    pub product: [Dword; 2],
    /// +0x10 (0x620468): not named (the qword's neighbour).
    pub unknown_10: [Dword; 2],
    /// +0x18 (0x620470): `dword_620470`: 0x40 or 0x80 by the caller of the ground search (`physics::count_ground_probe`, `physics::move_slot`, `physics::probe_ground`, `race::car_starts`).
    pub settle_mark: Dword,
}

/// The piece: the race block's bytes at IDA 0x620458 seen as `GroundProbe`.
pub struct InBlock;

impl core::ops::Deref for InBlock {
    type Target = GroundProbe;
    fn deref(&self) -> &GroundProbe {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const GroundProbe) }
    }
}

pub static GROUND_PROBE: InBlock = InBlock;

/// IDA 0x620458, the first byte of the piece.
pub const START: usize = 0x620458;
/// IDA 0x620474, the end of the piece.
pub const END: usize = 0x620474;

const _: () = assert!(END - START == core::mem::size_of::<GroundProbe>());

/// Where IDA 0x620458 is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(GroundProbe, product), 0x620460 - START);
        assert_eq!(offset_of!(GroundProbe, settle_mark), 0x620470 - START);
    }
}
