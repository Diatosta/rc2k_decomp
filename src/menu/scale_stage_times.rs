//! `menu::scale_stage_times` (0x46354E): the stage's four record times (`dword_75AAE8` ..
//! `dword_75AAF4`) scaled for the section (`section` held to 0 .. 5): the last three by
//! `flt_60463C`[section], the first by `flt_604654`[section] (`fild`, `fmul`, `fistp`; the
//! two factors kept in `flt_71C3D8` / `flt_71C3DC`); then logged ("stage times info", the
//! section, the record's +0x40, "a7x" and the first factor, "a8x" and the second, "times
//! a8-a5" and the four times).
//!
//! In:  edi the record
//! Out: eax the section held (no `pusha`); the rest kept
//!
//! Caller: `menu::calc_cpu_times`. Logs through the `logfile::` Rust versions (level 1).
//! Shadow mode runs the original alone (it logs). Self-test: the
//! original and the Rust version on a record of the test's, times and sections by lot, at
//! the test precisions (exact at 0x027F; the times within one at 0x037F, exact there too
//! below 2^29).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;

pub const TIMES_RVA: usize = rva(0x75AAE8); // dword_75AAE8 .. dword_75AAF4
const KEPT_RVA: [usize; 2] = [rva(0x71C3D8), rva(0x71C3DC)]; // flt_71C3D8, flt_71C3DC
const FACTORS_RVA: [usize; 2] = [rva(0x60463C), rva(0x604654)]; // flt_60463C, flt_604654: 6
const TEXTS_RVA: [usize; 4] = [rva(0x4DDF93), rva(0x4DDFA4), rva(0x4DDFA8), rva(0x4DDFAC)];
/// The section held to 0 .. 5.
pub fn held_section() -> u32 {
    rc2k_core::menu::stage_scale::held_section(unsafe { game_read(SECTION_RVA) })
}

/// The four times as scaled (see the header), and the two factors.
pub fn scaled(times: [u32; 4], section: u32) -> ([u32; 4], [u32; 2]) {
    let f: [u32; 2] = FACTORS_RVA.map(|r| unsafe { game_read(r + 4 * section as usize) });
    // Converted: `rc2k_core::menu::stage_scale` (exact at the game's precisions for times
    // below 2^29; its header).
    let out = rc2k_core::menu::stage_scale::scale_times(times, f.map(f32::from_bits));
    (out, f)
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn scale_stage_times(entry: &Regs) -> Result<Regs, Refused> {
    use crate::logfile::{log_float::log_float, log_int::log_int, log_write::log_write};
    let section = held_section();
    unsafe {
        let times: [u32; 4] = game_read(TIMES_RVA);
        let (times, f) = scaled(times, section);
        game_write(KEPT_RVA[0], f[0]);
        game_write(KEPT_RVA[1], f[1]);
        game_write(TIMES_RVA, times);
        let text = |n: usize| game_addr(TEXTS_RVA[n]) as *const u8;
        log_write(text(0), false, 1);
        log_int(section as i32, false, 1);
        log_int(get(entry.edi + 0x40) as i32, false, 1);
        log_write(text(1), false, 1);
        log_float(f32::from_bits(f[0]), 2, false, 1);
        log_write(text(2), false, 1);
        log_float(f32::from_bits(f[1]), 2, false, 1);
        log_write(text(3), false, 1);
        for (k, &t) in times.iter().enumerate() {
            log_int(t as i32, k == 3, 1);
        }
    }
    Ok(Regs {
        eax: section,
        ..*entry
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { scale_stage_times(r) }?))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}
