//! `drive::race_frame` (0x4541BF): one frame of the race. The window's messages
//! (`system::pump_messages`); quitting (`g_quitting` 1) ends it there. With the window
//! the active one and `dword_518380` not 1: the 3D scene begun (`scene::begin_scene`;
//! its carry: the frame lost), drawn (`sub_45429B`) and ended (`scene::end_scene`), the
//! race's sounds (`sound::race_sound_tick`, eax 0x12), the screenshot key (`sub_41F3DB`),
//! the 2D pass and the showing (`sub_4547AE`, carry: lost); made: carry clear. Lost (or
//! not drawn): unless quitting, the surfaces restored (`sub_405B10`, ebx `dword_518380`)
//! and the flags of `cmp (ebx ^ 1) | dword_518380, 1`.
//!
//! In:  nothing
//! Out: carry clear when the frame was made; otherwise the flags above (quitting: those
//!      of `cmp g_quitting, 1`); the registers as the calls leave them, ebx on the lost way
//!
//! Callers: `drive::race_loop`, `quit_question`, `pause_until_key`, `finish_run`,
//! `sub_4607A5`. The sound tick is its Rust version; the window, the scene, the
//! drawing (`sub_45429B`, `sub_4547AE`: not ported yet), the screenshot and the restore
//! through the game. A private copy (a call at +0). Ungated (the sound tick's quit jump
//! does not come back). Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose calls go to `menu::scripted`'s stand-ins, against the Rust version: each way.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::race_frame", rva(0x4541BF), thunk, &ORIGINAL)
    .private_copy(0x6A)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const CALLS: [usize; 8] = [0x0, 0x20, 0x27, 0x2C, 0x36, 0x3B, 0x40, 0x58];
const QUITTING_RVA: usize = rva(0x51837C); // g_quitting
const ACTIVE_RVA: usize = rva(0x518374); // g_windowActive
const LOST_RVA: usize = rva(0x518380); // dword_518380

/// The calls by place, as IDA shows them.
const CALLED: [usize; 9] = [
    0, 0x410535, // 1 system::pump_messages
    0x44E8A1, // 2 scene::begin_scene
    0x45429B, // 3 sub_45429B: the 3D drawing
    0x44E952, // 4 scene::end_scene
    0x421BA9, // 5 sound::race_sound_tick
    0x41F3DB, // 6 sub_41F3DB: the screenshot key
    0x4547AE, // 7 sub_4547AE: the 2D pass, the showing
    0x405B10, // 8 sub_405B10: the surfaces restored
];

/// A call by place (`CALLED`).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

/// The Rust version (see the header): the registers and flags it leaves.
pub fn race_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let cmp_one = |r: &mut Regs, v: u32| {
        r.set_carry(v < 1);
        r.set_zero(v == 1);
    };
    part(1, &mut r)?;
    if g(QUITTING_RVA) == 1 {
        // The game is closing. In the game its shutdown thread (`threading::request_quit`)
        // ends the program meanwhile; the standalone, which has none, ends it here.
        #[cfg(feature = "standalone")]
        crate::standalone::leave_from_race();
        #[cfg(not(feature = "standalone"))]
        {
            cmp_one(&mut r, 1);
            return Ok(r);
        }
    }
    'lost: {
        if g(ACTIVE_RVA) == 0 || g(LOST_RVA) == 1 {
            break 'lost;
        }
        part(2, &mut r)?;
        if r.carry() {
            break 'lost;
        }
        part(3, &mut r)?;
        part(4, &mut r)?;
        r.eax = 0x12;
        part(5, &mut r)?;
        part(6, &mut r)?;
        part(7, &mut r)?;
        if r.carry() {
            break 'lost;
        }
        r.set_carry(false);
        return Ok(r);
    }
    if g(QUITTING_RVA) == 1 {
        cmp_one(&mut r, 1);
        return Ok(r);
    }
    r.ebx = g(LOST_RVA);
    part(8, &mut r)?;
    r.ebx = (r.ebx ^ 1) | g(LOST_RVA);
    let v = r.ebx;
    cmp_one(&mut r, v);
    Ok(r)
}

/// In the game: the sound tick's Rust version, the rest through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 5 {
        use crate::sound::race_sound_tick::race_sound_tick_in;
        return race_sound_tick_in(r, &mut crate::sound::turn_parts::game_part);
    }
    unsafe { call_game(r, game_addr(rva(CALLED[place as usize]))) };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    // The thunk is ungated (a frame is long) and the Rust frame calls the Rust versions
    // directly, past every gate: one gate per frame, so that a `selftest`'s pause waits for
    // the frame under way and holds the next (as the audio thread's turns).
    crate::framework::hook_enter();
    let done = race_frame(r, &mut game_part);
    crate::framework::hook_leave();
    match done {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    use crate::menu::scripted::{Answer, begin, handler, notes};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x4541BF)),
            0x6A,
            &[0xE8, 0x71, 0xC3, 0xFB, 0xFF],
            &CALLS,
            &|c| {
                (1..9)
                    .find(|&p| rva(CALLED[p]) == c)
                    .map(|p| stand_in(p as u32))
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(handler);
    let vars = [at(QUITTING_RVA), at(ACTIVE_RVA), at(LOST_RVA)];
    let saved = capture(&vars);
    let carry = Answer {
        carry: true,
        ..Answer::default()
    };
    let regs = |r: &Regs| {
        [
            r.eax,
            r.ebx,
            r.ecx,
            r.edx,
            r.esi,
            r.edi,
            r.ebp,
            u32::from(r.carry()),
            u32::from(r.eflags & 0x40 != 0),
        ]
    };
    // (quitting, active, lost, begin's carry, the 2D pass's carry)
    for (quitting, active, lost, begin_fails, show_fails) in [
        (0u32, 1u32, 0u32, false, false),
        (1, 1, 0, false, false),
        (0, 0, 0, false, false),
        (0, 1, 1, false, false),
        (0, 1, 0, true, false),
        (0, 1, 0, false, true),
        (0, 1, 0, false, true),
    ] {
        let script = vec![
            (
                2,
                vec![if begin_fails {
                    carry
                } else {
                    Answer::default()
                }],
                Answer::default(),
            ),
            (
                7,
                vec![if show_fails { carry } else { Answer::default() }],
                Answer::default(),
            ),
            (
                8,
                vec![],
                Answer {
                    eax: None,
                    al: None,
                    carry: false,
                    zero: None,
                    write: None,
                },
            ),
        ];
        let prepare = || unsafe {
            game_write(QUITTING_RVA, quitting);
            game_write(ACTIVE_RVA, active);
            game_write(LOST_RVA, lost);
        };
        prepare();
        begin(&script);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (notes(), regs(&ro));
        prepare();
        begin(&script);
        let mine = race_frame(&Regs::SENTINEL, &mut |p, r| {
            handler(p, r);
            Ok(())
        });
        let mine = (notes(), mine.as_ref().map(regs).unwrap_or_default());
        t.check(theirs == mine, || {
            format!(
                "quitting {quitting}, active {active}, lost {lost}, begin fails {begin_fails}, show fails {show_fails}: calls {:x?} / {:x?}; registers {:x?} / {:x?}",
                theirs.0, mine.0, theirs.1, mine.1
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
