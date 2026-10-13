//! `scene::build_texture_catalogue` (0x42274B): gets memory for every texture that is
//! needed, making the textures smaller group by group until it is had.
//!
//! In:  nothing
//! Out: carry clear. The original leaves eax, ecx and edi changed; its caller sets
//!      them anew. The Rust version keeps them.
//!
//! The listing is one list of 432 steps of a few kinds, which
//! `scene/texture_steps.rs` holds as a table (made by `tools/texture_catalogue`):
//! log a text or a number; count a reduction in `dword_5DC97C` (0x100 counters);
//! raise the level a tag's textures are needed at (`scene::raise_texture_needs`: a
//! higher level is a smaller texture); try (`scene::try_allocate_textures`), which
//! ends the list when it succeeds; release what a failed try took
//! (`gfx::release_all_textures` 0x4051D3, Direct3D: not ported); and a group of steps
//! skipped when a flag is set.
//!
//! Before the list the counters are zeroed and the eight flags `dword_5DCD7C` ..
//! set: 1 for the first, 0 for the others. So the list reads: try as things are;
//! else your cars 2 .. 8 down from 256 to 128, try; and so on through the other
//! cars, the scenery, the sky, the front end, each time one group one step smaller.
//! After a successful try: "Allocated textures OK -", the needs counted once more
//! for the log, a rule. After the last failed one: "Cant allocate enough textures"
//! and fatal error 0x80600403.
//! One caller, `scene::set_up_textures`.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls go to the test, the try succeeding at
//! seven different places and never.

use crate::errors::{FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::redirect::RedirectedCopy;
use crate::scene::count_texture_needs::count_texture_needs_in_game;
use crate::scene::raise_texture_needs::raise_texture_needs;
use crate::scene::texture_steps::{CALLS, DONE, FAILED, STEPS};
use crate::scene::try_allocate_textures::try_allocate_textures_in_game;
use crate::service::harness::{Region, capture, restore};
use std::ffi::CStr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::build_texture_catalogue",
    rva(0x42274B),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const COUNTERS_RVA: usize = rva(0x5DC97C); // dword_5DC97C, 0x100 dwords
const COUNTERS_LEN: usize = 0x100 * 4;
/// `dword_5DCD7C` .. `dword_5DCD98`: which of the eight cars keep their size.
const FLAGS_RVA: usize = rva(0x5DCD7C);
const RELEASE_RVA: usize = rva(0x4051D3); // gfx::release_all_textures

const NOT_ENOUGH: FatalError = FatalError {
    code: 0x8060_0403,
    message: "",
};

/// A step of the list.
pub enum Step {
    Text(&'static CStr, bool),
    Number(u32, bool),
    Raise {
        level: u32,
        tag: u32,
    },
    /// One more reduction of this kind.
    Count(u32),
    Try,
    Release,
    CountNeeds,
    /// The steps, unless the flag is set.
    Unless(usize, &'static [Step]),
}

/// What the function calls.
pub enum Call {
    Text(&'static CStr, bool),
    Number(u32, bool),
    Raise {
        level: u32,
        tag: u32,
    },
    /// Answers whether the memory was had.
    Try,
    Release,
    CountNeeds,
}

// ---- Rust version ----

/// True when a try succeeded.
unsafe fn run(
    steps: &'static [Step],
    call: &mut dyn FnMut(Call) -> Result<bool, Refused>,
) -> Result<bool, Refused> {
    for step in steps {
        match *step {
            Step::Text(text, newline) => {
                call(Call::Text(text, newline))?;
            }
            Step::Number(number, newline) => {
                call(Call::Number(number, newline))?;
            }
            Step::Raise { level, tag } => {
                call(Call::Raise { level, tag })?;
            }
            Step::Count(index) => unsafe {
                let at = COUNTERS_RVA + index as usize * 4;
                game_write(at, game_read::<u32>(at).wrapping_add(1));
            },
            Step::Try => {
                if call(Call::Try)? {
                    return Ok(true);
                }
            }
            Step::Release => {
                call(Call::Release)?;
            }
            Step::CountNeeds => {
                call(Call::CountNeeds)?;
            }
            Step::Unless(flag, inner) => {
                if unsafe { game_read::<u32>(flag) } == 0 && unsafe { run(inner, call) }? {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

pub unsafe fn build_texture_catalogue(
    call: &mut dyn FnMut(Call) -> Result<bool, Refused>,
) -> Result<(), Refused> {
    unsafe {
        // The original does this after the first two log lines and the count, which
        // touch neither.
        std::ptr::write_bytes(game_addr(COUNTERS_RVA) as *mut u8, 0, COUNTERS_LEN);
        game_write(FLAGS_RVA, [1u32, 0, 0, 0, 0, 0, 0, 0]);
        if run(&STEPS, call)? {
            run(&DONE, call)?;
            return Ok(());
        }
        run(&FAILED, call)?;
        Err(NOT_ENOUGH.into())
    }
}

/// With the game's log, and the Rust versions of what it calls.
pub unsafe fn build_texture_catalogue_in_game() -> Result<(), Refused> {
    unsafe {
        build_texture_catalogue(&mut |call| {
            match call {
                Call::Text(text, newline) => log_write(text.as_ptr().cast(), newline, 1),
                Call::Number(number, newline) => log_int(number as i32, newline, 1),
                Call::Raise { level, tag } => {
                    raise_texture_needs(level, tag);
                }
                Call::Try => return Ok(try_allocate_textures_in_game()?),
                Call::Release => call_game(&mut Regs::default(), game_addr(RELEASE_RVA)),
                Call::CountNeeds => {
                    count_texture_needs_in_game()?;
                }
            }
            Ok(false)
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { build_texture_catalogue_in_game() } {
        Ok(()) => r.set_carry(false),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const START: usize = 0x42274B;
const LEN: usize = 0x1766;
const ENTRY_BYTES: [u8; 7] = [0x60, 0x8D, 0x05, 0x4F, 0xB1, 0x4D, 0x00];
/// The operand of the jump to the fatal stub (`jmp loc_4D90B3` at 0x423E89).
const FATAL_JUMP: usize = 0x423E8A - START;
/// The callees, by the place the test knows them under.
const CALLEES: [usize; 8] = [
    rva(0x401427), // logfile::log_write_l1
    rva(0x40142B), // logfile::log_write_l1_nl
    rva(0x401472), // logfile::log_int_l1
    rva(0x401476), // logfile::log_int_l1_nl
    rva(0x423F97), // scene::raise_texture_needs
    rva(0x4224C4), // scene::try_allocate_textures
    rva(0x4051D3), // gfx::release_all_textures
    rva(0x423EB1), // scene::count_texture_needs
];

/// 1 after the copy went to the fatal stub.
static WENT_FATAL: AtomicU32 = AtomicU32::new(0);

/// Where the copy's jump to the fatal stub lands: nothing is on the stack there but
/// the return address.
#[unsafe(naked)]
unsafe extern "C" fn landing() {
    core::arch::naked_asm!(
        "mov dword ptr [{flag}], 1",
        "ret",
        flag = sym WENT_FATAL,
    )
}

/// Each call: its place and what it was given (a text as a mark of its bytes).
static CALLED: Mutex<Vec<[u32; 3]>> = Mutex::new(Vec::new());
/// The try that succeeds (counted from 0), and how many have been made.
static TRIES: Mutex<(u32, u32)> = Mutex::new((0, 0));

fn text_mark(text: *const u8) -> u32 {
    unsafe { CStr::from_ptr(text.cast()) }
        .to_bytes()
        .iter()
        .fold(0x811C_9DC5u32, |mark, &byte| {
            (mark ^ byte as u32).wrapping_mul(0x0100_0193)
        })
}

fn handler(place: u32, r: &mut Regs) {
    let given = match place {
        0 | 1 => [place, text_mark(r.eax as *const u8), 0],
        2 | 3 => [place, r.eax, 0],
        4 => [place, r.eax, r.ebx],
        _ => [place, 0, 0],
    };
    CALLED.lock().unwrap().push(given);
    if place == 5 {
        let mut tries = TRIES.lock().unwrap();
        // The carry is clear when the memory was had.
        r.set_carry(tries.1 != tries.0);
        tries.1 += 1;
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(START)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                CALLEES
                    .iter()
                    .position(|&known| known == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    unsafe { copy.retarget(FATAL_JUMP, landing as *const () as usize) };
    let regions: [Region; 2] = [
        (game_addr(COUNTERS_RVA) as u32, COUNTERS_LEN),
        (game_addr(FLAGS_RVA) as u32, 0x20),
    ];
    let saved = capture(&regions);
    for succeeds in [0u32, 1, 2, 5, 20, 43, 44, u32::MAX] {
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            CALLED.lock().unwrap().clear();
            *TRIES.lock().unwrap() = (succeeds, 0);
            WENT_FATAL.store(0, Ordering::SeqCst);
            set_handler(handler);
            let failed = if rust {
                let done = build_texture_catalogue(&mut |call| {
                    let mut r = Regs::default();
                    let place = match call {
                        Call::Text(text, newline) => {
                            r.eax = text.as_ptr() as u32;
                            newline as u32
                        }
                        Call::Number(number, newline) => {
                            r.eax = number;
                            2 + newline as u32
                        }
                        Call::Raise { level, tag } => {
                            (r.eax, r.ebx) = (level, tag);
                            4
                        }
                        Call::Try => 5,
                        Call::Release => 6,
                        Call::CountNeeds => 7,
                    };
                    act(place, &mut r);
                    Ok(place == 5 && !r.carry())
                });
                match done {
                    Ok(()) => 0,
                    Err(Refused::Fatal(e)) if e.code == NOT_ENOUGH.code => 1,
                    Err(_) => 2,
                }
            } else {
                let mut r = Regs::SENTINEL;
                call_game(&mut r, copy.addr());
                let fatal = WENT_FATAL.load(Ordering::SeqCst);
                // A way out that is not the fatal one must leave the carry clear.
                if fatal == 0 && r.carry() { 3 } else { fatal }
            };
            (failed, capture(&regions), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine, || {
            let first = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "the try {succeeds:#x} succeeds: ended {} / {}; {} / {} calls, the first that differs {first:?} ({:x?} / {:x?}); memory same {}",
                theirs.0,
                mine.0,
                theirs.2.len(),
                mine.2.len(),
                first.map(|n| theirs.2[n]),
                first.map(|n| mine.2[n]),
                theirs.1 == mine.1
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
