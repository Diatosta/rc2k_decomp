//! `race::shadow_faces` (0x457618): the car's shadow on the course faces round it (see
//! `race/car_shadow.rs`): the last course record's faces, then every record in the 3 x 3
//! cells round the work car; each face below kind 0x10 and without bit 7 of its +2 word
//! as two triangles by `race::shadow_on_face` (`sub_4576C9`, covered here).
//!
//! In:  nothing
//! Out: ebx the record count (when any); the rest kept (a `retn` follows)
//!
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to a
//! copy of `sub_4576C9` whose face calls go to a stand-in that notes the corners, against
//! the Rust version noting the same; course records and faces of the test's (pointers
//! set), the car in cells round them; all put back.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::car_shadow::*;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::shadow_faces", rva(0x457618), thunk, &ORIGINAL).self_test(self_test);

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
    let done = unsafe { shadow_faces(r, &mut |e| shadow_on_face(e, &mut face_part).map(|_| ())) };
    match done {
        Ok(after) => r.ebx = after.ebx,
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

static CALLED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());

fn corners() -> [u32; 4] {
    unsafe {
        [
            game_read(rva(0x641938)),
            game_read(rva(0x64193C)),
            game_read(rva(0x641940)),
            game_read(rva(0x5CCCD0)),
        ]
    }
}

fn stand_in(_: u32, _: &mut Regs) {
    CALLED.lock().unwrap().push(corners());
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(stand_in);
    let inner = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4576C9,
            len: 0x73,
            entry: &[0x8B, 0x5E, 0x08, 0x83, 0xFB, 0xFF, 0x74, 0x6A],
            calls: &[0x46, 0x65],
            jumps_out: &[],
        },
        &|c| (c == rva(0x45773C)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let inner_addr = inner.addr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x457618,
            len: 0xB1,
            entry: &[0x60, 0x8B, 0x1D, 0x18, 0xCD, 0x5C, 0x00, 0x4B],
            calls: &[0x13, 0x9C],
            jumps_out: &[],
        },
        &|c| (c == rva(0x4576C9)).then_some(inner_addr),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    // Five records (cells round 0x10, 0x10), faces 0 .. 11 of kinds by index.
    let mut records = vec![0u32; 8 * 5];
    for (k, (x, z, first, last)) in [
        (0x10u32, 0x10u32, 0u32, 2u32),
        (0x11, 0x10, 3, 3),
        (0x13, 0x10, 4, 6),
        (0x10, 0x0F, 7, 8),
        (0x0F, 0x11, 9, 11),
    ]
    .into_iter()
    .enumerate()
    {
        records[8 * k] = x | z << 11;
        records[8 * k + 2] = first;
        records[8 * k + 3] = last;
    }
    records[8 * 3] = u32::MAX;
    let mut faces = vec![0u8; 0x40 * 12];
    for f in 0..12usize {
        faces[0x40 * f] = [0x01u8, 0x0F, 0x10, 0x05][f % 4];
        faces[0x40 * f + 2] = if f == 5 { 0x80 } else { 0 };
        for c in 0..4usize {
            let v = (100 * f + c) as u32;
            faces[0x40 * f + 4 + 4 * c..0x40 * f + 8 + 4 * c].copy_from_slice(&v.to_le_bytes());
        }
    }
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let kept = vec![
        at(rva(0x5CCD18)),
        at(rva(0x5CCCB8)),
        at(rva(0x5CCCBC)),
        g(0x641938, 0xC),
        at(rva(0x5CCCD0)),
        g(0x5CCBF0, 8),
        g(0x71A10C, 0xC),
    ];
    let saved = capture(&kept);
    let watch = vec![at(rva(0x5CCCD0)), g(0x5CCBF0, 8), g(0x641938, 0xC)];
    for car in [
        (0x10u32 << 21, 0x10u32 << 21),
        (0x12 << 21, 0x10 << 21),
        (0x40 << 21, 0x40 << 21),
    ] {
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x5CCD18), 5u32);
            game_write(rva(0x5CCCB8), records.as_ptr() as u32);
            game_write(rva(0x5CCCBC), faces.as_ptr() as u32);
            game_write(rva(0x71A10C), car.0 + 0x1234);
            game_write(rva(0x71A114), car.1 + 0x4321);
            CALLED.lock().unwrap().clear();
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            ro.ebx,
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(&watch),
        );
        prepare();
        let done = unsafe {
            shadow_faces(&Regs::SENTINEL, &mut |_| {
                CALLED.lock().unwrap().push(corners());
                Ok(())
            })
        };
        let mine = (
            done.as_ref().map(|r| r.ebx).unwrap_or(0),
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(&watch),
        );
        t.check(theirs == mine, || {
            format!("car {car:x?}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&kept, &saved);
    drop((records, faces, inner));
    t.finish()
}
