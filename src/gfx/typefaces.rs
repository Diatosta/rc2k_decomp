//! The fonts and the front end's tables, out of RAL.EXE's image (phase 5,
//! `notes/conversion.md`): IDA 0x51E3A4 .. 0x529DA5. The fonts (0x20 bytes each, from
//! 0x51E3C4: palette +0x14, sheet +0x18, as `gfx::build_fonts` hangs them), the icons'
//! palette, the fonts' colour palettes, the saved game's file buffer (0x51EBB4, 0x8000),
//! the saved games', replays' and set-ups' name lists, the replay's header (0x528504),
//! the paths and texts (`var\ral.dat`, `savedata\..`, the films', the fonts' sheets),
//! up to the driver names' key (0x529DA5, `settings::read_driver_names`).
//!
//! One block with the image's layout (`crate::store`), first values the image's
//! (`typefaces_data.rs`, generated). The code names the variables by IDA's address;
//! `framework::game_addr` and `moved::place` take such an address here. The menus' font
//! pointers are run-time addresses (made homes at start, `moved::point_into_homes`).

crate::block!(pub mod tables {
    start: 0x51E3A4,
    end: 0x529DA5,
    runs: crate::gfx::typefaces_data::RUNS,
});
