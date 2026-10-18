//! `race::copy_mirror_picture` (0x455DAD): the mirror's surface copied onto slot
//! `dword_5D0E08`'s with alpha 0xFF, pixel by pixel through the two formats (see
//! `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as the unlocks leave them (its caller goes on with a `mov`)
//!
//! The surfaces' lock and unlock are the platform's (`sub_41AA4B` / `sub_41AB02`,
//! `call_game`). Refuses a destination without its format (after the locks: it is known only
//! then; the original would fault). Caller: `race::forget_view_textures`. Shadow mode runs
//! the original alone (it writes a surface). Self-test: a copy whose locks and unlocks go to
//! stand-ins handing out test surfaces, against the Rust version with the same: 16 and 32
//! bits each side, test formats, a failed lock of each.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::gfx::pixel_format::TEST_FORMATS;
use crate::race::face_stage::compare_plain;
use crate::race::frame_views::{copy_mirror_picture, surface_part};
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::copy_mirror_picture", rva(0x455DAD), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { copy_mirror_picture(r, &mut surface_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The test's two surfaces: (pixels, pitch, bits) for slots 1 and 2, the dest format, and
/// which slot's lock fails (0 none).
static SOURCE: [AtomicU32; 3] = [AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0)];
static DEST: [AtomicU32; 3] = [AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0)];
static DEST_FORMAT: AtomicU32 = AtomicU32::new(0);
static FAILS: AtomicU32 = AtomicU32::new(0);
const W: u32 = 6;
const H: u32 = 3;

fn handler(place: u32, r: &mut Regs) {
    note(place, r);
    if place != 1 {
        return;
    }
    let fail = FAILS.load(Ordering::SeqCst) == r.eax;
    r.set_carry(fail);
    if fail {
        return;
    }
    let s = if r.eax == 1 { &SOURCE } else { &DEST };
    let [p, pitch, bits] = [0, 1, 2].map(|k| s[k].load(Ordering::SeqCst));
    unsafe {
        game_write(rva(0x4F49C0), p);
        game_write(rva(0x4F49BC), pitch);
        game_write(rva(0x4F49A8), bits);
        game_write(rva(0x4F49A0), W);
        game_write(rva(0x4F49A4), H);
        game_write(rva(0x4F49B8), DEST_FORMAT.load(Ordering::SeqCst));
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(handler);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x455DAD,
            len: 0xB4,
            entry: &[0xA1, 0x34, 0x42, 0x62, 0x00, 0xE8, 0x94, 0x4C],
            calls: &[0x05, 0x27, 0x64, 0x74, 0xA4, 0xAE],
            jumps_out: &[],
        },
        &|c| {
            [(0x41AA4Busize, 1u32), (0x41AB02, 2)]
                .iter()
                .find(|(a, _)| rva(*a) == c)
                .map(|(_, p)| crate::install::stand_ins::stand_in(*p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut source = vec![0u8; 0x100];
    for (k, b) in source.iter_mut().enumerate() {
        *b = (k as u8).wrapping_mul(97).wrapping_add(13);
    }
    let mut dest = vec![0xAAu8; 0x100];
    let d = dest.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![
        (d, 0x100),
        at(rva(0x6415EC)),
        at(rva(0x6415F0)),
        (game_addr(rva(0x4F49A0)) as u32, 0x24),
    ];
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x624234)),
        at(rva(0x5D0E08)),
        at(rva(0x4F4954)),
        (game_addr(rva(0x4F496C)) as u32, 0x14),
    ]);
    let saved = capture(&kept);
    for (screen_bits, dest_bits, formats, fails) in [
        (0x10u32, 0x10u32, (1usize, 4usize), 0u32),
        (0x20, 0x20, (0, 0), 0),
        (0x10, 0x20, (5, 2), 0),
        (0x20, 0x10, (3, 6), 0),
        (0x20, 0x20, (0, 1), 1),
        (0x10, 0x10, (4, 4), 2),
    ] {
        let line = compare_plain(
            &format!("bits {screen_bits} / {dest_bits} formats {formats:?} failing {fails}"),
            &|| unsafe {
                restore(&kept, &saved);
                ((d) as *mut u8).write_bytes(0xAA, 0x100);
                game_write(rva(0x624234), 1u32);
                game_write(rva(0x5D0E08), 2u32);
                game_write(rva(0x4F4954), screen_bits);
                (game_addr(rva(0x4F496C)) as *mut crate::gfx::pixel_format::PixelFormat)
                    .write_unaligned(TEST_FORMATS[formats.0]);
                for (k, v) in [source.as_ptr() as u32, W * 4 + 4, screen_bits]
                    .into_iter()
                    .enumerate()
                {
                    SOURCE[k].store(v, Ordering::SeqCst);
                }
                for (k, v) in [d, W * 4 + 8, dest_bits].into_iter().enumerate() {
                    DEST[k].store(v, Ordering::SeqCst);
                }
                DEST_FORMAT.store(
                    &TEST_FORMATS[formats.1] as *const _ as u32,
                    Ordering::SeqCst,
                );
                FAILS.store(fails, Ordering::SeqCst);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                copy_mirror_picture(r, &mut |p, r| {
                    handler(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    drop((source, dest));
    t.finish()
}
