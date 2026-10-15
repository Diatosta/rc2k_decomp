//! `audio::set_eax_room` (0x40D26B): the EAX room asked for set.
//!
//! In:  nothing
//! Out: eax = the room's entry in `g_eaxRooms`; every other register preserved
//!
//! See `audio/property_sets.rs`: the entry of `g_eaxRoom` (an EAX 1 reverb, 0x10 bytes)
//! given to `g_eaxProperties`' `Set` when that is there (its result not read). The `Set`
//! is a chunk (0x40D325) entered by `jmp`. Callers: `audio::update_voices` (twice),
//! `audio::make_buffer`. Refuses a room past the table's ten (the original reads past it).
//! Shadow mode runs the original alone. Self-test: redirected copies of the head and the
//! chunk (the head's jump retargeted) whose room and set are the test's, 30 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::property_sets::set_eax_room;
use crate::audio::test_sound::{Results, TestSound};
use crate::audio::voices::{EAX_ROOM_RVA, EAX_RVA, read};
use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::set_eax_room", rva(0x40D26B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// On the game's variables.
pub unsafe fn in_game() -> Result<u32, Refused> {
    unsafe {
        set_eax_room(
            read(game_addr(EAX_ROOM_RVA) as u32),
            game_addr(EAX_RVA) as u32,
        )
    }
}

unsafe fn rust(_: &Regs) -> Result<Leaves, Refused> {
    Ok(Leaves {
        eax: unsafe { in_game() }?,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // [0] the room, [1] the set.
    let mut vars = Box::new([0u32; 2]);
    let at = vars.as_mut_ptr() as u32;
    // The chunk, then the head with its jump to the chunk's copy.
    let copy = |address, len, entry: &[u8], variable: (usize, u32), references| {
        crate::audio::test_copy::make(
            &HOOK,
            &crate::audio::test_copy::Copy {
                address,
                len,
                entry,
                calls: &[],
                jumps_out: &[],
            },
            &|_| None,
            &[variable],
            references,
        )
    };
    let chunk = match copy(0x40D325, 0x25, &[0x60], (0x506C74, at + 4), 2) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let c = match copy(
        0x40D26B,
        0x14,
        &[0xA1, 0x78, 0x6C, 0x50, 0x00],
        (0x506C78, at),
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    // The `jmp` (E9) is at 0xE: mov 5 bytes, imul 3, lea 6.
    if unsafe { *((c.addr() + 0xE) as *const u8) } != 0xE9 {
        return t.skip("the head's jump is not where it was read");
    }
    unsafe { c.retarget(0xF, chunk.addr()) };
    let sound = TestSound::new();
    for room in 0..10u32 {
        for (there, set) in [(false, 0), (true, 0), (true, 0x8000_4005u32 as i32)] {
            let run = |rust: bool| {
                let p = if there { sound.property_set() } else { 0 };
                unsafe { (at as *mut [u32; 2]).write_volatile([room, p]) };
                sound.begin(Results {
                    set,
                    ..Results::default()
                });
                let before = Regs::SENTINEL;
                let eax = if rust {
                    unsafe { set_eax_room(room, at + 4) }.ok()
                } else {
                    let mut ro = before;
                    unsafe { call_game(&mut ro, c.addr()) };
                    ro.preserved_except(&before, EAX).then_some(ro.eax)
                };
                (eax, sound.noted())
            };
            let theirs = run(false);
            let mine = run(true);
            t.check(theirs.0.is_some() && theirs == mine, || {
                format!("room {room}, set there {there}, its result {set:#x}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    drop(vars);
    t.finish()
}
