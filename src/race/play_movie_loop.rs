//! `race::play_movie_loop` (0x4607A5): the viewer's replay played as a race: the skid
//! marks set up (`race::init_skidmarks`), "dpg replay", the block played from
//! (`dword_5C2530`) and the one it stops at (`dword_5C2538`) logged; the race's state 2
//! (`dword_5C5CE0`), the clock restarted (`scene::restart_clock`, `g_sceneClock` into
//! `dword_613840`); then each frame the input (`input::update`), a key pressed
//! (`g_keysPressed + 1`) ending it, the race's step (`drive::race_step`) and frame
//! (`drive::race_frame`), until the block played is the one to stop at; "dpg replay
//! exit".
//!
//! In:  nothing
//! Out: the registers as the steps leave them (eax `dword_5C2530` after a whole play); no
//!      flags read
//!
//! Caller: `race::play_movie`. Its calls through `part` (places in `CALLEES`; the logs 8
//! and 9); in the game their Rust versions, the frame through its hook (which gates each
//! frame, as for `drive::race_loop`). Ungated (it plays a replay). Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins (the frame moving the
//! block on, a key after some frames), against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::race::viewer::{
    BLOCK, PLAY_END, Part, Saved, answer, dispatch_parts, dword, gb, get, pb, put, region,
    run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::play_movie_loop", rva(0x4607A5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 10] = [
    0, 0x45857D, // 1 race::init_skidmarks
    0x401427, // 2 logfile::log_write_l1
    0x401476, // 3 logfile::log_int_l1_nl
    0x450381, // 4 scene::restart_clock
    0x420115, // 5 input::update
    0x4454D2, // 6 drive::race_step
    0x4541BF, // 7 drive::race_frame
    0x40142B, // 8 logfile::log_write_l1_nl
    0,
];
const CALLS: [usize; 11] = [
    0x0, 0xC, 0x19, 0x25, 0x32, 0x3E, 0x4E, 0x5D, 0x6B, 0x70, 0x89,
];

const STATE: usize = 0x5C5CE0; // dword_5C5CE0
const SCENE_CLOCK: usize = 0x613830; // g_sceneClock
const CLOCK_BASE: usize = 0x613840; // dword_613840
const KEYS_PRESSED: usize = 0x4F6CDD; // g_keysPressed + 1
const TEXTS: [usize; 4] = [0x4DDF45, 0x4DDF50, 0x4DDF61, 0x4DDF72]; // "dpg replay" ..

/// The Rust version: the registers it leaves.
pub fn play_movie_loop(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(1, &mut r)?;
    let log = |part: Part, r: &Regs, place: u32, eax: u32| -> Result<(), Refused> {
        let mut s = Regs { eax, ..*r };
        part(place, &mut s)
    };
    log(part, &r, 2, a(TEXTS[0]))?;
    log(part, &r, 2, a(TEXTS[1]))?;
    log(part, &r, 3, get(BLOCK))?;
    log(part, &r, 2, a(TEXTS[2]))?;
    log(part, &r, 3, get(PLAY_END))?;
    put(STATE, 2);
    part(4, &mut r)?;
    r.eax = get(SCENE_CLOCK);
    put(CLOCK_BASE, r.eax);
    loop {
        part(5, &mut r)?;
        if gb(KEYS_PRESSED) != 0 {
            break;
        }
        part(6, &mut r)?;
        part(7, &mut r)?;
        r.eax = get(BLOCK);
        if r.eax == get(PLAY_END) {
            break;
        }
    }
    log(part, &r, 2, a(TEXTS[3]))?;
    Ok(r)
}

/// In the game: the Rust versions, the frame through its hook.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::drive::race_parts;
    unsafe {
        match place {
            1 => *r = crate::race::stage_prefs::init_skidmarks(r),
            2 | 8 => crate::logfile::log_write::log_write(r.eax as *const u8, place == 8, 1),
            3 => crate::logfile::log_int::log_int(r.eax as i32, true, 1),
            4 => race_parts::restart_clock(r),
            5 => race_parts::input_update(r),
            6 => crate::drive::race_step::race_step(r)?.apply(r),
            _ => race_parts::race_frame(r),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        play_movie_loop(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places: &'static [(usize, u32)] = Box::leak(
        (1..9u32)
            .map(|p| (CALLEES[p as usize], p))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    // call race::init_skidmarks
    let copy = match crate::race::viewer::copy(
        0x4607A5,
        0x90,
        &[0xE8, 0xD3, 0x7D, 0xFF, 0xFF],
        &CALLS,
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [
        dword(BLOCK),
        dword(PLAY_END),
        dword(STATE),
        dword(CLOCK_BASE),
        dword(SCENE_CLOCK),
        region(KEYS_PRESSED, 1),
    ];
    let saved = Saved::new(&regions);
    // (block, end, a key after this many frames)
    for (block, end, key_after) in [(2u32, 5u32, 99u32), (2, 9, 3), (4, 5, 0)] {
        run_both(
            &mut t,
            &format!("block {block} to {end}, a key after {key_after}"),
            copy.addr(),
            Regs::SENTINEL,
            &regions,
            0,
            &|| {
                put(BLOCK, block);
                put(PLAY_END, end);
                put(STATE, 0);
                put(SCENE_CLOCK, 0x1234);
                pb(KEYS_PRESSED, 0);
                let mut frames = 0u32;
                answer(Box::new(move |place, r| match place {
                    5 => {
                        if frames == key_after {
                            pb(KEYS_PRESSED, 1);
                        }
                    }
                    7 => {
                        frames += 1;
                        put(BLOCK, get(BLOCK) + 1);
                        r.eax = 0x99;
                    }
                    _ => {}
                }));
            },
            &|r| play_movie_loop(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
