//! `race::load_game_map` (0x41B480): the in-race map (`LOADLEVEL_INIT : loadgamemap`).
//!
//! In:  eax = the map picture's path (`srcmaps\mapxx.pcx`), ebx = the key's
//!      (`srcmaps\keyxx.pcx`)
//! Out: with the packed file read, eax = `g_packedPath`; every other register preserved
//!
//! See `race/game_map.rs`. One caller, `sub_44426F` (loadlevel_init). A key not 8-bit is
//! fatal error 0x80E48073, raised from Rust. Shadow mode runs the original alone (it
//! allocates and reads). Self-test: a redirected copy whose `file_exists`, `read_sheet`
//! and `read_picture_file` are stand-ins (the sheet and picture of the test's, written
//! into the picture buffer; the packed case reads stage 21's real map), against the Rust
//! version with the same; compared with the pointers taken out, freed after each.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::race::game_map::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::load_game_map", rva(0x41B480), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { load_game_map(r, &mut game_part) } {
        Ok(g) => r.eax = g.eax,
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

/// The test's case: bit 0 the picture is "there", bit 1 the sheet read fails, bit 2 the
/// picture read fails.
static CASE: AtomicU32 = AtomicU32::new(0);

/// What the stand-ins do, for both sides: the sheet a few used blocks, the picture a
/// pattern, both into the picture buffer (16 MB).
fn stand_in(place: u32, r: &mut Regs) {
    use crate::gfx::picture::{FLAGS_RVA, PICTURE_RVA};
    let case = CASE.load(Ordering::SeqCst);
    unsafe {
        let buffer: u32 = game_read(PICTURE_RVA);
        match place {
            0 => r.set_carry(case & 1 == 0),
            1 => {
                if case & 2 != 0 {
                    r.set_carry(true);
                    return;
                }
                std::ptr::write_bytes(buffer as *mut u8, 0, 0x100_0000);
                for (x, y) in [
                    (40u32, 40u32),
                    (41, 40),
                    (300, 900),
                    (2040, 2047),
                    (0, 0),
                    (1000, 3),
                ] {
                    ((buffer + y * 0x2000 + x * 4) as *mut u32).write_unaligned(0x55);
                }
                game_write(FLAGS_RVA, game_read::<u32>(FLAGS_RVA) | 0x100);
                r.set_carry(false);
            }
            _ => {
                if case & 4 != 0 {
                    r.set_carry(true);
                    return;
                }
                let words = buffer as *mut u32;
                for n in 0..0x40_0000u32 {
                    words
                        .add(n as usize)
                        .write(n.wrapping_mul(0x9E37_79B9) >> 7);
                }
                r.set_carry(false);
            }
        }
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    stand_in(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::audio::test_copy::{Copy, make};
    use crate::gfx::pcx::{HEIGHT_LIMIT_RVA, WIDTH_LIMIT_RVA};
    use crate::gfx::picture::{FLAGS_RVA, HEIGHT_RVA, PICTURE_RVA, ROW_BYTES_RVA, WIDTH_RVA};
    use crate::install::stand_ins::{set_handler, stand_in as place};
    use crate::service::harness::at;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !lists_empty() {
        return t.skip("the map's lists hold blocks");
    }
    if !format_ready() {
        return t.skip("no pixel kinds yet");
    }
    set_handler(stand_in);
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x41B480,
            len: 0x305,
            entry: &[0x83, 0x3D, 0xB5, 0xE9, 0x5B, 0x00, 0x01],
            calls: &[
                0x09, 0x17, 0x36, 0x73, 0x8E, 0xF1, 0x1CD, 0x1E2, 0x250, 0x276, 0x2B9, 0x2F4,
            ],
            jumps_out: &[0x78, 0x93, 0x106],
        },
        &|c| match c {
            _ if c == rva(0x402EBE) => Some(place(0)),
            _ if c == rva(0x4119B7) => Some(place(1)),
            _ if c == rva(0x4119F4) => Some(place(2)),
            _ => None,
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut kept = crate::race::load_packed_game_map::kept();
    kept.extend([
        at(WIDTH_LIMIT_RVA),
        at(HEIGHT_LIMIT_RVA),
        at(PICTURE_RVA),
        at(WIDTH_RVA),
        at(HEIGHT_RVA),
        at(ROW_BYTES_RVA),
        at(FLAGS_RVA),
    ]);
    let saved = capture(&kept);
    let pictures = [
        at(WIDTH_LIMIT_RVA),
        at(HEIGHT_LIMIT_RVA),
        at(PICTURE_RVA),
        at(WIDTH_RVA),
        at(HEIGHT_RVA),
        at(ROW_BYTES_RVA),
    ];
    let picture = b"srcmaps\\map21.pcx\0";
    let key = b"srcmaps\\key21.pcx\0";
    for (what, packed, case) in [
        ("packed", 1u32, 0u32),
        ("built", 0, 0),
        ("built, the picture there", 1, 1),
        ("the sheet not read", 0, 2),
        ("the picture not read", 0, 4),
    ] {
        CASE.store(case, Ordering::SeqCst);
        let before = Regs {
            eax: picture.as_ptr() as u32,
            ebx: key.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let reset = || unsafe {
            restore(&kept, &saved);
            game_write(USE_PACKED_RVA, packed);
        };
        reset();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (
            snapshot(),
            capture(&pictures),
            [ro.eax, ro.ebx, ro.ecx, ro.edx, ro.esi, ro.edi, ro.ebp],
        );
        let _ = unsafe { free_game_map() };
        reset();
        let mine = unsafe { load_game_map(&before, &mut test_part) };
        let ours = (
            snapshot(),
            capture(&pictures),
            mine.as_ref()
                .map(|g| [g.eax, g.ebx, g.ecx, g.edx, g.esi, g.edi, g.ebp])
                .unwrap_or_default(),
        );
        let _ = unsafe { free_game_map() };
        t.check(mine.is_ok() && theirs == ours, || {
            let at = theirs.0.iter().zip(&ours.0).position(|(a, b)| a != b);
            format!(
                "{what}: rust {:?}; {} and {} values, first difference at {at:?}; pictures same {}; registers {:x?} / {:x?}",
                mine.err(),
                theirs.0.len(),
                ours.0.len(),
                theirs.1 == ours.1,
                theirs.2,
                ours.2
            )
        });
    }
    restore(&kept, &saved);
    t.finish()
}
