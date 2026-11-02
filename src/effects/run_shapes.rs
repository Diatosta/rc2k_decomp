//! `effects::run_shapes` (0x4BDF72): the mask shapes' frame: `effects::pick_shapes`,
//! `effects::turn_all_shapes`; then when `g_sceneClock` reaches `dword_844E30`, the next
//! time (the clock plus a random's low byte, `dword_89C994`) and whether the shapes move
//! (`dword_844E2C`: another random, turned right 16, plus the clock, `and`
//! `dword_844E34[settings +0x2748]`, not 0); while they move, the shape index
//! (`dword_84460C`) from a running count (`dword_82E38A` plus `g_marksFrameTicks` by
//! `flt_89C9A0`, or by `flt_89C99C` when the settings' +0x274C is 1 and the index >> 3 is
//! not 0), its bits 16 .. 23, and `pick_shapes` again.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C1148`. Refuses null settings, a +0x2748 of 9 or more when it is read
//! (`dword_844E34` holds 9), and its steps' refusals; on a refusal nothing stays written
//! (all it may write is put back).

use crate::effects::emitters::next_c5;
use crate::effects::mask_constants::MASK_CONSTANTS;
use crate::effects::mask_picks::PICKS;
use crate::effects::mask_vars::MASKS;
use crate::effects::pick_shapes::pick_shapes;
use crate::effects::setup::CLOCK;
use crate::effects::shape_points::turn_all_shapes;
use crate::errors::{BadInput, Refused};

use crate::math::x87;
use crate::physics::tables_store::PHYSICS;
const MASK_COUNT: u32 = 9;

/// The Rust version (see the header).
pub fn run_shapes() -> Result<(), Refused> {
    let marks = crate::effects::setup::marks()?;
    {
        pick_shapes()?;
        turn_all_shapes()?;
        let clock: u32 = PHYSICS.scene_clock.bits();
        if clock as i32 >= PICKS.next.get() as i32 {
            let r = next_c5(&MASK_CONSTANTS.time_random);
            PICKS
                .next
                .set((r & 0xFF).wrapping_add(PHYSICS.scene_clock.bits()));
            let mask = marks.listener_switch_7.get();
            if mask >= MASK_COUNT {
                return Err(BadInput("a mask number past dword_844E34's 9").into());
            }
            let moving = next_c5(&MASK_CONSTANTS.move_random)
                .rotate_right(16)
                .wrapping_add(clock)
                & PICKS.masks[mask as usize].get();
            PICKS.moving.set(1);
            if moving == 0 {
                PICKS.moving.set(0);
                return Ok(());
            }
        }
        if PICKS.moving.get() == 0 {
            return Ok(());
        }
        let rate = if marks.listener_switch_5.get() == 1 {
            (PICKS.index.get() >> 3 != 0).then_some(&MASK_CONSTANTS.rates[0])
        } else {
            Some(&MASK_CONSTANTS.rates[1])
        };
        if let Some(rate) = rate {
            let count = x87::int_times_plus_int_to_int(
                CLOCK.frame_ticks.get() as i32,
                rate.bits(),
                MASKS.count.get() as i32,
            );
            MASKS.count.set(count as u32);
            PICKS.index.set((count as u32 >> 16) & 0xFF);
        }
        pick_shapes()?;
    }
    Ok(())
}
