//! The 3D card's tables and the stage's picture state, out of RAL.EXE's image (phase 5,
//! `notes/conversion.md`): IDA 0x5ABA8C .. 0x5BE9BD. The texture formats' kinds and
//! descriptions, the video and system memory texture records and their counts, the
//! sizes, the game fonts' list and heap size, the `nofmv` switch, the plate texture, the
//! map's lists, tiles and paths, the packed maps' switches. Up to `g_packedPath`
//! (0x5BE9BD, `files`).
//!
//! One block with the image's layout (`crate::store`), first values the image's
//! (`records_data.rs`, generated). The code names the variables by IDA's address;
//! `framework::game_addr` takes such an address here.

crate::block!(pub mod tables {
    start: 0x5ABA8C,
    end: 0x5BE9BD,
    runs: crate::gfx::records_data::RUNS,
});
