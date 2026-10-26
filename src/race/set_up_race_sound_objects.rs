//! `race::set_up_race_sound_objects` (0x415C93): the race's sound objects set up: the
//! sound tables made ready (`race::prepare_sound_tables`), the players' and objects'
//! counts, the players' objects, each object's first dword -1 .. -8 and its state reset;
//! then `race::restart_race_sounds`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sound_tables.rs`. Seven callers (`sub_4843EC` and menus; a call follows each).
//! It begins with its call at +1: private copy, both calls relocated, manual patch.
//! Refuses, before anything, a chosen car past 7 or a sound object missing. Shadow mode
//! runs the original alone (the audio thread reads the objects). Self-test: a copy whose
//! two calls go to stand-ins and whose eleven references to the objects and their table
//! are the test's, against the Rust version on the same: one and two players, mode 2, a
//! chosen car.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::sound_tables::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::set_up_race_sound_objects",
    rva(0x415C93),
    thunk,
    &ORIGINAL,
)
.private_copy(0x213)
.relocate_calls(&[0x1, 0x20C])
.manual_patch()
.self_test(self_test);

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
    if let Err(e) = unsafe { set_up_race_sound_objects(r, &ObjectVars::game(), &mut object_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { set_up_race_sound_objects(r, &ObjectVars::game(), &mut object_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::audio::test_copy::{Copy, make};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let objects: Vec<Vec<u32>> = (0..8)
        .map(|n| vec![0x5A5A_0000 + n; OBJECT_LEN / 4])
        .collect();
    let table: Vec<u32> = objects.iter().map(|o| o.as_ptr() as u32).collect();
    let v = ObjectVars {
        table: table.as_ptr() as u32,
        objects: [0, 1, 2, 3, 4, 5, 6, 7].map(|n| table[n]),
    };
    let mut patches: Vec<(usize, u32)> = vec![(0x512A10, v.table)];
    patches.extend(OBJECTS_IDA.iter().zip(v.objects).map(|(&a, o)| (a, o)));
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x415C93,
            len: 0x213,
            entry: &[0x60, 0xE8, 0x9D, 0x79, 0xFF, 0xFF],
            calls: &[0x1, 0x20C],
            jumps_out: &[],
        },
        &|c| match c {
            _ if c == rva(0x40D636) => Some(crate::install::stand_ins::stand_in(PREPARE)),
            _ if c == rva(0x40D34A) => Some(crate::install::stand_ins::stand_in(RESTART)),
            _ => None,
        },
        &patches,
        11,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::stand_ins::set_handler(crate::race::level_init::note);
    let mut regions: Vec<Region> = object_regions();
    let inputs = [0x75ACE8usize, 0x75AE20, 0x75ACEC, 0x75AF8C].map(crate::service::harness::at);
    regions.extend(inputs);
    let kept = regions.clone();
    let saved = capture(&kept);
    regions.extend(objects.iter().map(|o| (o.as_ptr() as u32, OBJECT_LEN)));
    for (players, mode, one, chosen) in [
        (1u32, 0u32, 0u32, 0u32),
        (2, 0, 0, 0),
        (2, 2, 0, 0),
        (1, 2, 1, 5),
        (2, 0, 1, 7),
    ] {
        crate::race::mud::run_both(
            &mut t,
            &format!("{players} players, mode {mode}, one {one}, car {chosen}"),
            &copy,
            &regions,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0x5A, len);
                }
                game_write(rva(0x75ACE0), players);
                game_write(rva(0x75ACD8), mode);
                game_write(rva(0x52A910), one);
                game_write(rva(0x52A920), chosen);
            },
            &|r| unsafe { set_up_race_sound_objects(r, &v, &mut crate::race::mud::test_part) },
        );
    }
    restore(&kept, &saved);
    drop((objects, table));
    t.finish()
}
