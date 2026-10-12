//! `gfx::load_picture` (0x411863): loads a picture file into the picture buffer at the
//! screen's size.
//!
//! In:  eax = path, which may be written to (see `files::swap_missing_extension`)
//! Out: carry = the file could not be read; every register preserved
//! Sets the picture's size to 800 x 600 with rows that follow one another (row length
//! 0xFFFFFFFD), clears the buffer and calls `read_picture_file`. Whatever that says, the
//! picture's size then becomes the screen's (`dword_4F494C` x `dword_4F4950`) and its row
//! length four bytes a pixel. If the file was read and its decoded width is not the
//! screen's, `rescale_picture` makes it so.
//! Ten callers, `menu::load_background` and `service::load_background` among them.
//! Shadow mode runs the original alone: everything it calls is compared in its own hook.
//! Refuses what the functions it calls refuse. The name is a reading.

use crate::errors::Refused;
use crate::files::stream::LastPath;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::clear_picture::clear_picture;
use crate::gfx::fill_row_table::{ROW_TABLE_ROOM, ROW_TABLE_RVA};
use crate::gfx::pcx::{DECODED_WIDTH_RVA, reader_inputs, reader_regions};
use crate::gfx::picture::{
    FLAGS_RVA, HEIGHT_RVA, PICTURE_DWORDS, ROW_BYTES_RVA, WIDTH_RVA, picture,
};
use crate::gfx::picture_as_surface::{SURFACE_DWORDS, SURFACE_RVA};
use crate::gfx::png::{INFO_RVA, PngInfo};
use crate::gfx::read_picture_file::read_picture_file;
use crate::gfx::rescale_picture::rescale_picture;
use crate::gfx::surface::CLIP_LEFT_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::load_picture", rva(0x411863), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const SCREEN_WIDTH_RVA: usize = rva(0x4F494C); // dword_4F494C
pub const SCREEN_HEIGHT_RVA: usize = rva(0x4F4950); // dword_4F4950

// ---- Rust version ----

/// Returns whether the file was read.
pub unsafe fn load_picture(path: *mut u8) -> Result<bool, Refused> {
    unsafe {
        // Refused before anything is written, if there is no buffer.
        if picture().is_null() {
            clear_picture()?;
        }
        game_write(WIDTH_RVA, 800u32);
        game_write(HEIGHT_RVA, 600u32);
        game_write(ROW_BYTES_RVA, 0xFFFF_FFFDu32);
        clear_picture()?;
        let read = read_picture_file(path);
        let screen_width: u32 = game_read(SCREEN_WIDTH_RVA);
        game_write(WIDTH_RVA, screen_width);
        game_write(HEIGHT_RVA, game_read::<u32>(SCREEN_HEIGHT_RVA));
        game_write(ROW_BYTES_RVA, screen_width << 2);
        if !read? {
            return Ok(false);
        }
        if game_read::<u32>(DECODED_WIDTH_RVA) != screen_width {
            rescale_picture()?;
        }
        Ok(true)
    }
}

// ---- hook ----

/// The picture's size and row length as the function leaves them, whatever else happens.
unsafe fn size_is_the_screens() -> bool {
    unsafe {
        let screen_width: u32 = game_read(SCREEN_WIDTH_RVA);
        game_read::<u32>(WIDTH_RVA) == screen_width
            && game_read::<u32>(HEIGHT_RVA) == game_read::<u32>(SCREEN_HEIGHT_RVA)
            && game_read::<u32>(ROW_BYTES_RVA) == screen_width << 2
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        match unsafe { load_picture(r.eax as *mut u8) } {
            Ok(read) => r.set_carry(!read),
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if !unsafe { size_is_the_screens() } || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| {
            "the original left the picture's size other than the screen's".to_string()
        });
    }
    r.eflags = o.eflags;
}

// ---- for this self-test and `menu::load_background`'s: everything a picture's loading
// writes, saved and put back, and files to load ----

pub struct LoaderTest {
    regions: Vec<Region>,
    saved: Vec<u8>,
    last_path: LastPath,
}

pub const TEST_PNG: &str = "var\\rc2k_hook_load_a.png";
pub const TEST_PCX: &str = "var\\rc2k_hook_load_b.pcx";
/// A small PNG (rescaled when loaded), a PCX, a PNG that is not there but whose PCX is, and
/// one with neither.
pub const TEST_PATHS: [&[u8]; 4] = [
    b"var\\rc2k_hook_load_a.png",
    b"var\\rc2k_hook_load_b.pcx",
    b"var\\rc2k_hook_load_b.png",
    b"var\\rc2k_hook_load_c.png",
];

impl LoaderTest {
    /// Why the test cannot run, or the test.
    pub fn begin() -> Result<Self, &'static str> {
        if picture().is_null() {
            return Err("the picture buffer is not allocated");
        }
        // The buffer is known to hold a screen of 800 x 600; with another screen the
        // rescaler would write more than the test saves.
        if unsafe {
            (
                game_read::<u32>(SCREEN_WIDTH_RVA),
                game_read::<u32>(SCREEN_HEIGHT_RVA),
            )
        } != (800, 600)
        {
            return Err("the screen is not 800 x 600");
        }
        let made = std::fs::write(
            TEST_PNG,
            crate::gfx::png::test_png(14, 6, 3, &[0, 1, 4], 2).0,
        )
        .and_then(|_| {
            std::fs::write(
                TEST_PCX,
                crate::gfx::read_pcx_24bit::test_file(11, 5, 3, 12, 6),
            )
        });
        if made.is_err() {
            return Err("cannot create a test file in the game's var folder");
        }
        let regions = [
            &reader_regions()[..],
            &reader_inputs(),
            &[
                at(FLAGS_RVA),
                at(crate::gfx::decode_png::FILE_RVA),
                at(crate::gfx::decode_png::FILE_LEN_RVA),
                (game_addr(rva(0xA8C190)) as u32, 0x14),
                (game_addr(INFO_RVA) as u32, std::mem::size_of::<PngInfo>()),
                (game_addr(rva(0x4F509C)) as u32, 0x20),
                (game_addr(ROW_TABLE_RVA) as u32, ROW_TABLE_ROOM as usize * 4),
                (game_addr(SURFACE_RVA) as u32, SURFACE_DWORDS * 4),
                (game_addr(CLIP_LEFT_RVA) as u32, 16),
                (picture() as u32, PICTURE_DWORDS * 4),
            ],
        ]
        .concat();
        Ok(Self {
            saved: capture(&regions),
            regions,
            last_path: LastPath::save(),
        })
    }

    /// The same start for both sides.
    pub fn prepare(&self) {
        let written = reader_regions();
        restore(&written, &vec![0xAA; written.iter().map(|r| r.1).sum()]);
        unsafe {
            std::ptr::write_bytes(picture(), 0x5A, PICTURE_DWORDS * 4);
            std::ptr::write_bytes(game_addr(rva(0x4F509C)) as *mut u8, 0xAA, 0x20);
            game_write(FLAGS_RVA, 0u32);
            game_write(crate::gfx::pcx::WIDTH_LIMIT_RVA, 0x800u32);
            game_write(crate::gfx::pcx::HEIGHT_LIMIT_RVA, 0x800u32);
        }
    }

    /// What a loading leaves that is the same on both sides: the picture, its variables,
    /// the surface and the clip rectangle, the rescaler's variables.
    pub fn observed(&self) -> Vec<u8> {
        capture(&[
            at(WIDTH_RVA),
            at(HEIGHT_RVA),
            at(ROW_BYTES_RVA),
            at(FLAGS_RVA),
            (game_addr(rva(0x4F509C)) as u32, 0x20),
            (game_addr(SURFACE_RVA) as u32, SURFACE_DWORDS * 4),
            (game_addr(CLIP_LEFT_RVA) as u32, 16),
            (picture() as u32, PICTURE_DWORDS * 4),
        ])
    }

    pub fn end(self) {
        restore(&self.regions, &self.saved);
        self.last_path.restore();
        let _ = std::fs::remove_file(TEST_PNG);
        let _ = std::fs::remove_file(TEST_PCX);
    }
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = match LoaderTest::begin() {
        Ok(test) => test,
        Err(why) => return t.skip(why),
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    for path in TEST_PATHS {
        let run = |rust: bool| {
            test.prepare();
            let mut buffer = path.to_vec();
            buffer.push(0);
            let before = Regs {
                eax: buffer.as_mut_ptr() as u32,
                ..Regs::SENTINEL
            };
            let read = if rust {
                unsafe { load_picture(buffer.as_mut_ptr()) }.ok()
            } else {
                let mut ro = before;
                unsafe { call_game(&mut ro, original) };
                ro.preserved_except(&before, 0).then_some(!ro.carry())
            };
            (read, buffer, test.observed())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0.is_some(), || {
            let first = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "path={:?}: original read={:?} left the path {:?}; rust read={:?} left {:?}; first different byte of what they left {first:?}",
                String::from_utf8_lossy(path),
                theirs.0,
                String::from_utf8_lossy(&theirs.1),
                mine.0,
                String::from_utf8_lossy(&mine.1)
            )
        });
    }
    test.end();
    t.finish()
}
