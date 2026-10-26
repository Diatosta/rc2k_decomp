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
use crate::marks::clip::CLIP;
use crate::marks::kind_values::MARKS_KIND;
use crate::marks::setup::CLOCK;
use crate::marks::store::game_addr;
use crate::marks::tuning::{MARKS_FLOATS, MARKS_FRAME, MARKS_KIND_FLOATS};
use crate::marks::work_store::MARKS_WORK;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::session::SCREEN;

const KIND_AT: u32 = 0x26A8; // the settings' kind
const LIVE_AT: u32 = 0x18; // the settings' dword set to 1
/// The table's entries; the kinds go to 0x21 (the other kinds' tables have 0x22).
const KIND_FLOATS: u32 = 0x17;
const KINDS: u32 = rc2k_core::marks::kind_values::KINDS;
/// 18240.0, the value of most of the table's kinds: the kinds past it get it (section 16).
const KIND_FLOAT_PAST: u32 = 0x468E_8000;

/// The Rust version (see the header): `settings` the settings (the original's esi), `seed`
/// the seed (eax). The steps (`nullsub_1` before each, `nullsub_26` for step 7: nothing).
pub fn init_marks(settings: u32, seed: u32) -> Result<(), Refused> {
    use crate::marks::*;
    {
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        if GamePtr::new(settings + KIND_AT).read::<u32>("a mark's field")? >= KINDS {
            return Err(BadInput("a marks kind of 0x22 or more").into());
        }
        crate::marks::setup::SETTINGS.pointer.set(settings);
        MARKS_WORK.seed.set(seed);
        CLOCK.started.set(0);
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
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        GamePtr::new(settings + LIVE_AT).write::<u32>(1, "a mark's field")?;
        for r in &CLOCK.settings_copies {
            r.set(1);
        }
        setup::tick_clock()?;
        MARKS_FRAME.steps.set_bits(0u32);
        let less: u32 = MARKS_FLOATS.one.bits();
        for (n, side) in [&SCREEN.width, &SCREEN.height].into_iter().enumerate() {
            let half = side.get() >> 1;
            // The screen's middle x, it less `flt_76516C`, the same for y.
            CLIP.edges[2 * n].set_bits(x87::from_int(half as i32));
            CLIP.edges[2 * n + 1].set_bits(x87::int_less(half as i32, less));
        }
        let kind: u32 = MARKS_KIND.kind.bits();
        // Fixed (section 16, 2026-10-23): the original reads past the table for kinds
        // 0x17 .. 0x21 (the next variables: `g_marksKindFloat` itself, a 0/1 flag, 9, ..),
        // so those cars' marks lose their depth bias.
        let float = if kind < KIND_FLOATS {
            MARKS_KIND_FLOATS.kind_floats[kind as usize].bits()
        } else {
            KIND_FLOAT_PAST
        };
        MARKS_KIND_FLOATS.kind_float.set_bits(float);
    }
    Ok(())
}
