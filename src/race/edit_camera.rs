//! The two "edit a camera" actions of the replay viewer, and the camera changes they lead
//! to (IDA 2026-10-19):
//! - `race::edit_camera_1` (0x41E599), `race::edit_camera_2` (0x41E5BA): the replay kept
//!   for an undo (`race::keep_undo`); unless the block shown (`dword_5C2530`) is the last
//!   (`dword_5BECD0`), `dword_96E480` 1 or 2: the keys then change that camera.
//! - `race::next_camera` (0x41E5EB), `race::previous_camera` (0x41E654): unless the block
//!   shown is the last, its camera byte `dword_96E480 - 1` (`unk_963BB3[edit + block * 8]`)
//!   one on (past 0x2E back to 0) or back (below 0 to 0x2E), skipping the cameras whose
//!   name (`word_5C291C`) is 0x1C1 or 0x1C2 unless the replay is one player's
//!   (`dword_719B34` 1) with a first car outside 0x17 .. 0x20; then the change spread on
//!   (`race::spread_camera`, dh the old value).

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::physics::flow::a;
use crate::race::viewer::{
    BLOCK, BLOCKS, CAMERA_EDIT, CAMERA_NAMES, CAMERAS, FIRST_CAR, ONE_PLAYER, Saved, TABLE_LEN,
    dword, fill, get, put, region,
};
use crate::service::harness::Region;

/// `edit_camera_1` / `_2`: eax the block shown.
pub fn edit_camera(entry: &Regs, camera: u32) -> Regs {
    let mut r = *entry;
    crate::race::keep_undo::keep_undo();
    r.eax = get(BLOCK);
    if r.eax != get(BLOCKS) {
        put(CAMERA_EDIT, camera);
    }
    r
}

fn name_of(camera: u8) -> u16 {
    unsafe { (a(CAMERA_NAMES + 2 * camera as usize) as *const u16).read_unaligned() }
}

/// `next_camera` (`step` 1) and `previous_camera` (-1): the registers they leave.
pub unsafe fn step_camera(entry: &Regs, step: i8) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = get(BLOCK);
    if r.eax == get(BLOCKS) {
        return Ok(r);
    }
    r.ebx = a(0x963BB3)
        .wrapping_add(get(CAMERA_EDIT))
        .wrapping_add(r.eax.wrapping_mul(8));
    let at = r.ebx as *mut u8;
    let old = unsafe { at.read() };
    // Every camera for one player's replay unless the first car is 0x17 .. 0x20.
    let any = get(ONE_PLAYER) == 1 && !(0x17..0x21).contains(&get(FIRST_CAR));
    if !any && (0..=0x2E).all(|n| matches!(name_of(n), 0x1C1 | 0x1C2)) {
        return Err(BadInput("every camera is skipped: the original goes round for ever").into());
    }
    let mut dl = old;
    loop {
        dl = if step > 0 {
            let n = dl.wrapping_add(1);
            if (n as i8) > 0x2E { 0 } else { n }
        } else {
            let n = dl.wrapping_sub(1);
            if (n as i8) >= 0 { n } else { 0x2E }
        };
        if any {
            break;
        }
        r.eax = dl as u32;
        if !matches!(name_of(dl), 0x1C1 | 0x1C2) {
            break;
        }
    }
    unsafe { at.write(dl) };
    r.edx = r.edx & !0xFFFF | (old as u32) << 8 | dl as u32;
    let spread = unsafe { crate::race::spread_camera::spread_camera(&r) };
    if spread.is_err() {
        unsafe { at.write(old) }; // refused before it wrote: nothing left written
    }
    spread
}

/// What the camera changes may write: the cameras and the edit switch.
pub fn camera_regions() -> Vec<Region> {
    vec![region(CAMERAS - 1, TABLE_LEN + 1), dword(CAMERA_EDIT)]
}

/// The inputs the tests set (saved and put back by the caller).
pub fn test_inputs() -> Vec<Region> {
    let mut all = camera_regions();
    all.extend([
        dword(BLOCK),
        dword(BLOCKS),
        dword(ONE_PLAYER),
        dword(FIRST_CAR),
        region(CAMERA_NAMES, 0x60),
    ]);
    all.extend(crate::race::keep_undo::regions());
    all
}

/// One test setting: the cameras filled, the block, the count and the header's values.
pub fn set_cameras(block: u32, blocks: u32, edit: u32, one_player: u32, car: u32) {
    fill(CAMERAS, TABLE_LEN, 7);
    // A run of equal cameras after the block, for the spreading.
    for n in block..(block + 4).min(0x708) {
        crate::race::viewer::pb(CAMERAS + 8 * n as usize, 0x2C);
        crate::race::viewer::pb(CAMERAS + 8 * n as usize + 1, 0x01);
    }
    put(BLOCK, block);
    put(BLOCKS, blocks);
    put(CAMERA_EDIT, edit);
    put(ONE_PLAYER, one_player);
    put(FIRST_CAR, car);
}

/// Saved over a test.
pub fn save() -> Saved {
    Saved::new(&test_inputs())
}

/// What `edit_camera_1` / `_2` may write.
pub fn edit_regions() -> Vec<Region> {
    let mut all = camera_regions();
    all.extend(crate::race::keep_undo::regions());
    all
}

/// The self-test of `edit_camera_1` / `_2`: the block the last or not.
pub fn test_edit(
    hook: &crate::framework::RawHook,
    call_original: crate::screens::harness::CallOriginal,
    rust: crate::race::viewer::Rust,
) -> (usize, usize) {
    let Some(mut t) = crate::framework::SelfTest::begin(hook) else {
        return (0, 0);
    };
    let saved = save();
    for (block, blocks) in [(3u32, 9u32), (9, 9), (0, 1)] {
        crate::race::viewer::check_original(
            &mut t,
            &format!("block {block} of {blocks}"),
            call_original,
            Regs::SENTINEL,
            &edit_regions(),
            0,
            &|| set_cameras(block, blocks, 0, 0, 0),
            rust,
        );
    }
    drop(saved);
    t.finish()
}

/// The self-test of `next_camera` / `previous_camera`: cameras 1 and 2, cameras at the
/// ends of the range, skipped cameras on the way (the names set by the test), one player's
/// replay with cars either side, the block the last.
pub fn test_step(
    hook: &crate::framework::RawHook,
    call_original: crate::screens::harness::CallOriginal,
    rust: crate::race::viewer::Rust,
) -> (usize, usize) {
    use crate::race::viewer::pb;
    let Some(mut t) = crate::framework::SelfTest::begin(hook) else {
        return (0, 0);
    };
    let saved = save();
    // Names: of every five cameras one 0x1C1, one 0x1C2, the rest not skipped.
    let names = || {
        for n in 0..0x30usize {
            let name: u16 = match n % 5 {
                0 => 0x1C1,
                1 => 0x1C2,
                _ => 0x100 + n as u16,
            };
            for (k, b) in name.to_le_bytes().into_iter().enumerate() {
                pb(CAMERA_NAMES + 2 * n + k, b);
            }
        }
    };
    for (block, blocks, edit, first, one, car) in [
        (3u32, 9u32, 1u32, 0x2Cu8, 0u32, 0u32),
        (3, 9, 2, 0x01, 0, 0),
        (3, 9, 1, 0x2E, 0, 0),
        (3, 9, 1, 0x00, 0, 0),
        (3, 9, 1, 0x04, 0, 0),
        (3, 9, 2, 0x10, 1, 0x18),
        (3, 9, 1, 0x10, 1, 0x16),
        (3, 9, 1, 0x2E, 1, 0x21),
        (9, 9, 1, 0x10, 0, 0),
        (8, 9, 2, 0x20, 0, 0),
    ] {
        let prepare = || {
            names();
            set_cameras(block, blocks, edit, one, car);
            pb(CAMERAS + 8 * block as usize + edit as usize - 1, first);
        };
        crate::race::viewer::check_original(
            &mut t,
            &format!(
                "block {block} of {blocks}, camera {edit} at {first:#x}, one {one} car {car:#x}"
            ),
            call_original,
            Regs::SENTINEL,
            &camera_regions(),
            0,
            &prepare,
            rust,
        );
    }
    drop(saved);
    t.finish()
}
