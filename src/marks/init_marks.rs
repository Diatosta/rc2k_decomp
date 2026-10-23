//! `marks::init_marks` (0x4D12F7): the marks set up for a stage: the settings and the
//! seed kept, then the fifteen steps (each after `nullsub_1` with its number in eax), the
//! screen's middle and the kind's float.
//!
//! In:  esi the settings, eax the seed
//! Out: eax the kind's float (bits), ebp the kind, esi the settings; the rest as
//!      `marks::tick_clock` leaves them
//!
//! One caller, init mark's end (0x457030). Shadow mode runs the original alone (its steps
//! take memory and read files). Refuses (section 15) no settings, and a kind (the
//! settings' +0x26A8, which step 1 makes `g_marksKind`) past `dword_765240`'s 0x17
//! entries; both before anything. Self-test: a redirected copy whose 30 calls go to
//! stand-ins noting every register, against the Rust version on the same stand-ins
//! (settings of the test's; three kinds and two screen sizes); put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

const KIND_AT: u32 = 0x26A8; // the settings' kind
const LIVE_AT: u32 = 0x18; // the settings' dword set to 1
const CLOCK_STARTED_RVA: usize = rva(0x764FC8); // g_marksClockStarted
const ONES_RVA: [usize; 2] = [rva(0x764FF0), rva(0x764FF4)]; // dword_764FF0, dword_764FF4
const CLEARED_RVA: usize = rva(0x76513C); // dword_76513C
const SCREEN_WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const SCREEN_HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
/// flt_7687E4 .. flt_7687F0: the screen's middle x, it less `flt_76516C`, the same for y.
const MIDDLE_RVA: usize = rva(0x7687E4);
const LESS_RVA: usize = rva(0x76516C); // flt_76516C
const KIND_FLOATS_RVA: usize = rva(0x765240); // dword_765240: 0x17 floats by kind
const KINDS: u32 = 0x17;
const KIND_FLOAT_RVA: usize = rva(0x76529C); // flt_76529C
/// The place of `nullsub_1` (before each step, eax the step's number).
const NUMBERED: u32 = 16;
const STEPS: u32 = 15;

/// The Rust version (see the header). Places 1 .. 15 the steps (7 `nullsub_26`), 16
/// `nullsub_1`.
pub unsafe fn init_marks(entry: &Regs) -> Result<Regs, Refused> {
    use crate::marks::lines::SETTINGS_RVA;
    let mut g = *entry;
    unsafe {
        if g.esi == 0 {
            return Err(BadInput("no marks settings").into());
        }
        if ((g.esi + KIND_AT) as *const u32).read_unaligned() >= KINDS {
            return Err(BadInput("a marks kind past dword_765240's entries").into());
        }
        game_write(SETTINGS_RVA, g.esi);
        game_write(crate::marks::setup::SEED_RVA, g.eax);
        game_write(CLOCK_STARTED_RVA, 0u32);
        for step in 1..STEPS {
            g.eax = step;
            crate::marks::init_marks::game_part(NUMBERED, &mut g)?;
            crate::marks::init_marks::game_part(step, &mut g)?;
        }
        g.eax = STEPS;
        crate::marks::init_marks::game_part(NUMBERED, &mut g)?;
        g.esi = game_read(SETTINGS_RVA);
        ((g.esi + LIVE_AT) as *mut u32).write_unaligned(1);
        for r in ONES_RVA {
            game_write(r, 1u32);
        }
        crate::marks::init_marks::game_part(STEPS, &mut g)?;
        game_write(CLEARED_RVA, 0u32);
        let less: u32 = game_read(LESS_RVA);
        for (n, side) in [SCREEN_WIDTH_RVA, SCREEN_HEIGHT_RVA]
            .into_iter()
            .enumerate()
        {
            g.eax = game_read::<u32>(side) >> 1;
            let to = MIDDLE_RVA + 8 * n;
            game_write(to, x87::from_int(g.eax as i32));
            game_write(to + 4, x87::int_less(g.eax as i32, less));
        }
        g.ebp = game_read(crate::marks::kind_values::KIND_RVA);
        g.eax = game_read(KIND_FLOATS_RVA + 4 * g.ebp as usize);
        game_write(KIND_FLOAT_RVA, g.eax);
    }
    Ok(g)
}

/// In the game: the steps' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::*;
    unsafe {
        let left = match place {
            1 => edges::load_kind_data(r)?,
            2 => setup::make_wave_tables(r)?,
            3 => profiles::make_profiles(r, game_addr(profiles::RESUME_RVA) as u32)?,
            4 => setup::seed_noise(r),
            5 => setup::make_noise_curve(r)?,
            6 => blocks::allocate_places(r, &mut blocks::game_part)?,
            8 => kind_values::set_kind_values(r, &kind_values::FIRST)?,
            9 => kind_values::set_kind_values(r, &kind_values::SECOND)?,
            10 => load_mask_pictures::load_mask_pictures(r, &mut load_mask_pictures::game_part)?,
            11 => setup::clear_slots(r),
            12 => set_up_surface::set_up_surface(r, &mut set_up_surface::game_part)?,
            13 => blocks::allocate_ring(r, &mut blocks::game_part)?,
            14 => lines::set_up_blocks(r)?,
            15 => setup::tick_clock(r),
            // `nullsub_26`, `nullsub_1`.
            _ => return Ok(()),
        };
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        };
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { init_marks(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
