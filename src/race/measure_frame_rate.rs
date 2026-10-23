//! `race::measure_frame_rate` (0x42009E): every 16th call (`dword_613838` counting, back to 0
//! at 0x10) the time-stamp counter (`rdtsc`) into `dword_6214E8`; with the cycles since the
//! last time (`dword_6214EC`, into `dword_62400C`) above 0 (signed), the frame rate
//! `flt_6214F0` = the CPU's clock `dword_5183E8` x `flt_5C5D18` x `flt_5C5D18` x
//! `flt_5C5D08` / the cycles (x87, as the original); the time kept for the next.
//!
//! In:  nothing
//! Out: unchanged but on the 16th: eax and edx the counter, ebx the cycles; no flags read
//!
//! Caller: `sub_4547AE` (0x4547FD); `flt_6214F0` is shown by `race::draw_frame_rate`.
//! The counter through `tsc` (in the game `rdtsc`). Self-test: a copy of the original
//! (its own `rdtsc`), then the Rust version given the value the copy read; counts before,
//! at and past the 16th, cycles above and below 0, at the test precisions (the rate
//! within `FLOAT` at extended). The Rust version is `measure_frame_rate_plain`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, game_write, rva};

const COUNT_RVA: usize = rva(0x613838); // dword_613838
const NOW_RVA: usize = rva(0x6214E8); // dword_6214E8
const LAST_RVA: usize = rva(0x6214EC); // dword_6214EC
const RATE_RVA: usize = rva(0x6214F0); // flt_6214F0
const CYCLES_RVA: usize = rva(0x62400C); // dword_62400C, a scratch

/// The rate worked out in plain floats.
// Converted: `rc2k_core::race::frame_rate::frame_rate`.
fn frame_rate_plain() {
    unsafe {
        let f = |a: usize| f32::from_bits(game_read(rva(a)));
        let rate = rc2k_core::race::frame_rate::frame_rate(
            game_read(rva(0x5183E8)),
            f(0x5C5D18),
            f(0x5C5D08),
            game_read(CYCLES_RVA),
        );
        game_write(RATE_RVA, rate.to_bits());
    }
}

/// The converted version: the hook's and `race::show_frame`'s (the exact x87 version went
/// once its one Rust caller took this): the registers it leaves; `tsc` gives the counter's
/// low and high halves.
pub unsafe fn measure_frame_rate_plain(entry: &Regs, tsc: &mut dyn FnMut() -> (u32, u32)) -> Regs {
    // Converted: `rc2k_core::race::frame_rate::{count_frame, cycles_since}`.
    use rc2k_core::race::frame_rate::{count_frame, cycles_since};
    let mut r = *entry;
    unsafe {
        let (count, due) = count_frame(game_read(COUNT_RVA));
        game_write(COUNT_RVA, count);
        if !due {
            return r;
        }
        (r.eax, r.edx) = tsc();
        game_write(NOW_RVA, r.eax);
        let last: u32 = game_read(LAST_RVA);
        r.ebx = r.eax.wrapping_sub(last);
        if let Some(cycles) = cycles_since(r.eax, last) {
            game_write(CYCLES_RVA, cycles);
            frame_rate_plain();
        }
        r.eax = game_read(NOW_RVA);
        game_write(LAST_RVA, r.eax);
    }
    r
}

/// The time-stamp counter, as `rdtsc` gives it.
pub fn rdtsc() -> (u32, u32) {
    let t = unsafe { core::arch::x86::_rdtsc() };
    (t as u32, (t >> 32) as u32)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    // Shadow mode runs the original alone: two reads of the counter never agree.
    *r = Regs {
        esp: r.esp,
        ..unsafe { measure_frame_rate_plain(r, &mut rdtsc) }
    };
    Ok(())
}
