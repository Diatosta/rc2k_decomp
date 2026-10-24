//! `saves::load_options` (0x4AAB32): reads the game's kept options, `var\ral.dat`.
//!
//! In:  nothing
//! Out: carry = not loaded; every register preserved
//! `g_optionsLoaded` (0x75557D) = 0. Up to 0x10000 bytes of the file into the scratch block
//! at 0x93E8B0; it is taken if it starts "R5CI" and has 0x50004 at +8. From +0x10 on it is
//! copied out piece by piece to the 69 places of `OPTIONS` (0xB656 bytes: the screen
//! modes, the sound's and the network's settings, the four players, the record
//! table, the players' names and much else). After the last piece the file must have
//! "####", else fatal error 0x80ACFF01 "ral.dat out of date" (the pieces have been copied
//! by then). Then `saves::rebuild_car_lists`, and `g_optionsLoaded` = 1.
//! A file shorter than that is not noticed as such: what the scratch block held decides.
//! One caller, `sub_411239`. `sub_4AAE57` writes the file, the same pieces the other way.
//! Shadow mode reads twice and compares every place. Self-test: a redirected copy that
//! reads files of the test's and copies the pieces into memory of the test's, so that
//! the game's own options are not touched.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::read_file::read_file;
use crate::framework::{game_addr, rva};
use crate::saves::rebuild_car_lists::rebuild_car_lists;
use crate::settings::{TEXT_ROOM, TEXT_RVA};

// ---- game data ----

const PATH_RVA: usize = rva(0x5295D8); // "var\ral.dat"
pub const LOADED_RVA: usize = rva(0x75557D); // g_optionsLoaded
/// The file's pieces after its head, in order: (where each goes, its length).
#[rustfmt::skip]
pub const OPTIONS: [(u32, usize); 69] = [
    (0x518BB0, 0x4),
    (0x4F4944, 0x4),
    (0x4F4948, 0x4),
    (0x4F50D0, 0x4),
    (0x4F82C4, 0x4),
    (0x4F82C8, 0x4),
    (0x4F82D8, 0x4),
    (0x5180F0, 0x4),
    (0x5180F4, 0x4),
    (0x5180F8, 0x4),
    (0x5180FC, 0x4),
    (0x518100, 0x4),
    (0x52AB90, 0x40),
    (0x52ABD0, 0x4),
    (0x52ABE4, 0x40),
    (0x52AC24, 0x4),
    (0x52AC94, 0x10),
    (0x51EBB0, 0x4),
    (0x527834, 0x4),
    (0x528500, 0x4),
    (0x5BEC08, 0x20),
    (0x5BEC28, 0x4),
    (0x4F66DD, 0x4),
    (0x4F66E1, 0x4),
    (0x4F66E5, 0x4),
    (0x4F66E9, 0x4),
    (0x755A4C, 0x28C),
    (0x755CD8, 0x28C),
    (0x755F64, 0x28C),
    (0x7561F0, 0x28C),
    (0x755684, 0x4),
    (0x755688, 0x4),
    (0x75568C, 0x4),
    (0x755690, 0x4),
    (0x755694, 0x4),
    (0x755698, 0x4),
    (0x75569C, 0x4),
    (0x7556A0, 0x4),
    (0x755640, 0x2C),
    (0x75566C, 0x18),
    (0x755254, 0x4),
    (0x7554C8, 0x4),
    (0x75ACCC, 0x4),
    (0x758800, 0x6),
    (0x755258, 0xA0),
    (0x7552F8, 0x50),
    (0x735576, 0x2A48),
    (0x739642, 0x47C),
    (0x7391C2, 0x47C),
    (0x738D42, 0x47C),
    (0x7388C2, 0x47C),
    (0x738442, 0x47C),
    (0x737FC2, 0x47C),
    (0x739AC2, 0x47C),
    (0x739F42, 0x47C),
    (0x73A3C2, 0x47C),
    (0x73A842, 0x47C),
    (0x73ACC2, 0x47C),
    (0x73B142, 0x47C),
    (0x742CB0, 0xC30),
    (0x7438E0, 0xC30),
    (0x744510, 0xC30),
    (0x745140, 0xC30),
    (0x745D70, 0xC30),
    (0x7469A0, 0xC30),
    (0x755784, 0x20),
    (0x7557A4, 0x20),
    (0x7557C4, 0x20),
    (0x7557E4, 0x20),
];
const PIECES_LEN: usize = 0xB656;
const _: () = {
    let mut total = 0;
    let mut n = 0;
    while n < OPTIONS.len() {
        total += OPTIONS[n].1;
        n += 1;
    }
    assert!(total == PIECES_LEN);
};

// ---- Rust version ----

/// Where the function works: the game's own places, or a test's.
pub struct Places<'a> {
    pub loaded: *mut u32,
    /// Where the piece with this number goes.
    pub piece: &'a dyn Fn(usize) -> *mut u8,
    pub rebuild_car_lists: &'a mut dyn FnMut() -> Result<(), BadInput>,
}

/// True if the options were loaded.
pub unsafe fn load_options(path: *const u8, places: &mut Places) -> Result<bool, Refused> {
    unsafe {
        places.loaded.write_unaligned(0);
        let file = game_addr(TEXT_RVA) as *mut u8;
        let read = read_file(path, file, TEXT_ROOM + 1, 0)?;
        // Converted: `rc2k_core::saves::kept` (`head_ok`, `pieces`), over the scratch block
        // the file was read into (what it held past the file decides, as in the original).
        use rc2k_core::saves::kept;
        let block = std::slice::from_raw_parts(file, TEXT_ROOM as usize + 1);
        if !read.ok || !kept::head_ok(block, kept::OPTIONS) {
            return Ok(false);
        }
        let sizes = OPTIONS.map(|(_, len)| len);
        let (pieces, ended) = kept::pieces(block, &sizes)?;
        for (n, piece) in pieces.into_iter().enumerate() {
            std::ptr::copy(piece.as_ptr(), (places.piece)(n), piece.len());
        }
        if !ended {
            return Err(FatalError {
                code: 0x80AC_FF01,
                message: "ral.dat out of date",
            }
            .into());
        }
        (places.rebuild_car_lists)()?;
        places.loaded.write_unaligned(1);
        Ok(true)
    }
}

/// `load_options` on the game's own file and places.
pub unsafe fn load_game_options() -> Result<bool, Refused> {
    unsafe {
        load_options(
            game_addr(PATH_RVA) as *const u8,
            &mut Places {
                loaded: game_addr(LOADED_RVA) as *mut u32,
                piece: &|n| crate::moved::place(OPTIONS[n].0 as usize) as *mut u8,
                rebuild_car_lists: &mut || rebuild_car_lists(),
            },
        )
    }
}
