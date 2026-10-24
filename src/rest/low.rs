//! The image's lower scattered variables, out of RAL.EXE's image (phase 5,
//! `notes/conversion.md`): the ranges of 0x4E0000 .. 0x51E384 between the blocks moved
//! before (the log's texts and paths, the file modes, the locale's lists, the keyboard
//! files' tail, the graphics settings, the screen modes' choices, the sound sets, the
//! audio's and the session's leftovers, the fonts' palettes, ...), and three more pieces:
//! 0x5AB4B4 .. 0x5ABA18 (the 3D cards' and modes' list), 0x5BEAC0 .. 0x5BEC30 (the tyre
//! change's floats and screen parts) and 0x764F2C .. 0x764F40 (after the HUD's block).
//!
//! Blocks with the image's layout (`crate::store`), first values the image's
//! (`low_data.rs`, generated); in `low` the ranges that had moved before keep their bytes
//! unused, and `crate::moved::MOVED` lists only `OWN`. The code names the variables by
//! IDA's address (`framework::game_addr` takes such an address here); the tyre screen's
//! inline asm reads `tyres` as `[{ty} + RVA]` with `ty = tyres_asm_base()`.

crate::block!(pub mod low {
    start: 0x4E0000,
    end: 0x51E384,
    runs: crate::rest::low_data::LOW_RUNS,
});

crate::block!(pub mod cards {
    start: 0x5AB4B4,
    end: 0x5ABA18,
    runs: crate::rest::low_data::CARDS_RUNS,
});

crate::block!(pub mod tyres {
    start: 0x5BEAC0,
    end: 0x5BEC30,
    runs: crate::rest::low_data::TYRES_RUNS,
});

crate::block!(pub mod hud_end {
    start: 0x764F2C,
    end: 0x764F40,
    runs: crate::rest::low_data::HUD_END_RUNS,
});

/// The ranges that are this module's own (IDA start, end).
pub const OWN: [(usize, usize); 39] = [
    (0x4E0000, 0x4E0094),
    (0x4E00A8, 0x4E0140),
    (0x4E014C, 0x4E0544),
    (0x4E0558, 0x4E0578),
    (0x4E0594, 0x4E0598),
    (0x4E059C, 0x4E05A4),
    (0x4E06EA, 0x4E073A),
    (0x4E073E, 0x4E0758),
    (0x4E075C, 0x4E0798),
    (0x4E07C8, 0x4E07D4),
    (0x4E07D8, 0x4E07F8),
    (0x4F0DC8, 0x4F3034),
    (0x4F3044, 0x4F32B4),
    (0x4F32BC, 0x4F3870),
    (0x4F3874, 0x4F3B74),
    (0x4F492C, 0x4F494C),
    (0x4F4984, 0x4F49C8),
    (0x4F50C0, 0x4F51F0),
    (0x4F52C4, 0x4F66DD),
    (0x4F6714, 0x4F673A),
    (0x4F683D, 0x4F68D8),
    (0x4F7E18, 0x4F7FD8),
    (0x506E6C, 0x50A7A4),
    (0x511AE4, 0x511B0C),
    (0x511B10, 0x5124E4),
    (0x512508, 0x512A10),
    (0x514F90, 0x516358),
    (0x51639C, 0x51655C),
    (0x5168C0, 0x517F40),
    (0x518120, 0x518370),
    (0x51839C, 0x51839F),
    (0x5184A8, 0x5184F4),
    (0x518934, 0x518998),
    (0x518BA8, 0x519118),
    (0x51B9C8, 0x51DE2C),
    (0x51DE30, 0x51E384),
    (0x5AB4B4, 0x5ABA18),
    (0x5BEAC0, 0x5BEC30),
    (0x764F2C, 0x764F40),
];

/// Where IDA's address `ida` of the blocks is.
pub fn home(ida: usize) -> usize {
    let at = |start: usize, base: usize| base + (ida - start);
    if ida >= hud_end::START {
        at(hud_end::START, hud_end::base())
    } else if ida >= tyres::START {
        at(tyres::START, tyres::base())
    } else if ida >= cards::START {
        at(cards::START, cards::base())
    } else {
        at(low::START, low::base())
    }
}

/// What an RVA of `tyres` is added to in inline asm (`[{ty} + RVA]`).
pub fn tyres_asm_base() -> u32 {
    (tyres::base() as u32).wrapping_sub(crate::framework::rva(tyres::START) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_ranges_are_in_the_blocks_and_moved() {
        let blocks = [
            (low::START, low::END),
            (cards::START, cards::END),
            (tyres::START, tyres::END),
            (hud_end::START, hud_end::END),
        ];
        for (start, end) in OWN {
            assert!(blocks.iter().any(|&(s, e)| s <= start && end <= e));
            assert_eq!(crate::moved::place(start), home(start));
            assert_eq!(crate::moved::place(end - 1), home(end - 1));
        }
    }
}
