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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::scene::count_texture_needs::count_texture_needs_in_game;
use crate::scene::raise_texture_needs::raise_texture_needs;
use crate::scene::texture_steps::{DONE, FAILED, STEPS};
use crate::scene::try_allocate_textures::try_allocate_textures_in_game;
use std::ffi::CStr;

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
                Call::Release => crate::calls::call(RELEASE_RVA, &mut Regs::default()),
                Call::CountNeeds => {
                    count_texture_needs_in_game()?;
                }
            }
            Ok(false)
        })
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    match unsafe { build_texture_catalogue_in_game() } {
        Ok(()) => r.set_carry(false),
        Err(e) => return Err(e),
    }
    Ok(())
}
