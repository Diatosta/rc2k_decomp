//! The scattered rest of the image's buffers, out of RAL.EXE's image (phase 5,
//! `notes/conversion.md`): the ranges between the blocks moved before, from 0x93CCC8 (the
//! end of the initialised data) to the driver names (0x95ECF8). One block with the image's
//! layout (`crate::store`), first values the image's (`data.rs`, generated); the ranges
//! of it that had moved before (the numbers' text, the log's line, the numbered paths,
//! the lexicon's work, the report, ...) keep their bytes unused, and `crate::moved::MOVED`
//! lists only the ranges below (`OWN`). The code
//! names the variables by IDA's address; `framework::game_addr` takes such an address here.
//! Not moved, because nothing reads them: 0xA8C18C (a dword) and 0xA8C1A4 .. 0xA8C5B0
//! (zlib's own variables; the PNG decoder is Rust's).

pub mod data;
pub mod low;
pub mod low_data;
pub mod rdata;
pub mod rdata_data;

crate::block!(pub mod scratch {
    start: 0x93CCC8,
    end: 0x95ECF8,
    runs: crate::rest::data::SCRATCH_RUNS,
});

/// The ranges that are this module's own (IDA start, end).
pub const OWN: [(usize, usize); 6] = [
    (0x93CCC8, 0x93E030), // zlib's messages
    (0x93E430, 0x93E830), // `text`, the strings' buffer
    (0x93E8B0, 0x94E8B0), // `g_pngInfo` and the settings' scratch (0x10000)
    (0x94EAB0, 0x94EAB4), // a dword
    (0x952AF8, 0x9566F8), // `g_mixBuffer`
    (0x95E6F8, 0x95ECF8), // the installer's and network's text fields
];

/// Where IDA's address `ida` of the block is.
pub fn home(ida: usize) -> usize {
    scratch::base() + (ida - scratch::START)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_ranges_are_in_the_blocks_and_moved() {
        for (start, end) in OWN {
            assert!(scratch::START <= start && end <= scratch::END);
            assert_eq!(crate::moved::place(start), home(start));
            assert_eq!(crate::moved::place(end - 1), home(end - 1));
        }
    }
}
