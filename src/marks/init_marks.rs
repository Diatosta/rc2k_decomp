//! `marks::init_marks` (0x4D12F7): the marks set up for a stage: the settings and the
//! seed kept, then the fifteen steps (each after `nullsub_1` with its number in eax), the
//! screen's middle and the kind's float.
//!
//! In:  the settings, the seed (the original's esi, eax; its ecx and edi are read by no
//!      step)
//! Out: nothing (no caller reads the registers it leaves)
//!
//! One caller, init mark's end (0x457030). Refuses no settings, and a kind (the settings'
//! +0x26A8, which step 1 makes `g_marksKind`) of 0x22 or more (the kinds' tables); both
//! before anything. Kinds 0x17 .. 0x21 are past `g_marksKindFloatTable`'s 0x17 entries:
//! fixed (section 16, 2026-10-23), they get 18240.0.

use crate::errors::{BadInput, Refused};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;
use crate::session::SCREEN;

const KIND_AT: u32 = 0x26A8; // the settings' kind
const LIVE_AT: u32 = 0x18; // the settings' dword set to 1
const CLOCK_STARTED_RVA: Slot = Slot::at(0x764FC8); // g_marksClockStarted
const ONES_RVA: [Slot; 2] = [Slot::at(0x764FF0), Slot::at(0x764FF4)]; // dword_764FF0, dword_764FF4
const CLEARED_RVA: Slot = Slot::at(0x76513C); // dword_76513C
/// flt_7687E4 .. flt_7687F0: the screen's middle x, it less `flt_76516C`, the same for y.
const MIDDLE_RVA: Slot = Slot::at(0x7687E4);
const LESS_RVA: Slot = Slot::at(0x76516C); // flt_76516C
const KIND_FLOATS_RVA: Slot = Slot::at(0x765240); // g_marksKindFloatTable: 0x17 floats by kind
/// The table's entries; the kinds go to 0x21 (the other kinds' tables have 0x22).
const KIND_FLOATS: u32 = 0x17;
const KINDS: u32 = rc2k_core::marks::kind_values::KINDS;
/// 18240.0, the value of most of the table's kinds: the kinds past it get it (section 16).
const KIND_FLOAT_PAST: u32 = 0x468E_8000;
const KIND_FLOAT_RVA: Slot = Slot::at(0x76529C); // flt_76529C

/// The Rust version (see the header): `settings` the settings (the original's esi), `seed`
/// the seed (eax). The steps (`nullsub_1` before each, `nullsub_26` for step 7: nothing).
pub unsafe fn init_marks(settings: u32, seed: u32) -> Result<(), Refused> {
    use crate::marks::lines::SETTINGS_RVA;
    use crate::marks::*;
    unsafe {
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        if ((settings + KIND_AT) as *const u32).read_unaligned() >= KINDS {
            return Err(BadInput("a marks kind of 0x22 or more").into());
        }
        game_write(SETTINGS_RVA, settings);
        game_write(crate::marks::setup::SEED_RVA, seed);
        game_write(CLOCK_STARTED_RVA, 0u32);
        edges::load_kind_data()?;
        setup::make_wave_tables()?;
        profiles::make_profiles(game_addr(profiles::RESUME_RVA) as u32)?;
        setup::seed_noise();
        setup::make_noise_curve()?;
        blocks::allocate_places()?;
        kind_values::set_kind_values(&kind_values::FIRST)?;
        kind_values::set_kind_values(&kind_values::SECOND)?;
        load_mask_pictures::load_mask_pictures()?;
        setup::clear_slots();
        set_up_surface::set_up_surface()?;
        blocks::allocate_ring()?;
        lines::set_up_blocks()?;
        let settings: u32 = game_read(SETTINGS_RVA);
        ((settings + LIVE_AT) as *mut u32).write_unaligned(1);
        for r in ONES_RVA {
            game_write(r, 1u32);
        }
        setup::tick_clock();
        game_write(CLEARED_RVA, 0u32);
        let less: u32 = game_read(LESS_RVA);
        for (n, side) in [&SCREEN.width, &SCREEN.height].into_iter().enumerate() {
            let half = side.get() >> 1;
            let to = MIDDLE_RVA + 8 * n;
            game_write(to, x87::from_int(half as i32));
            game_write(to + 4, x87::int_less(half as i32, less));
        }
        let kind: u32 = game_read(crate::marks::kind_values::KIND_RVA);
        // Fixed (section 16, 2026-10-23): the original reads past the table for kinds
        // 0x17 .. 0x21 (the next variables: `g_marksKindFloat` itself, a 0/1 flag, 9, ..),
        // so those cars' marks lose their depth bias.
        let float = if kind < KIND_FLOATS {
            game_read(KIND_FLOATS_RVA + 4 * kind as usize)
        } else {
            KIND_FLOAT_PAST
        };
        game_write(KIND_FLOAT_RVA, float);
    }
    Ok(())
}
