//! `effects::run_shapes` (0x4BDF72): the mask shapes' frame: `effects::pick_shapes`,
//! `effects::turn_all_shapes`; then when `g_sceneClock` reaches `dword_844E30`, the next
//! time (the clock plus a random's low byte, `dword_89C994`) and whether the shapes move
//! (`dword_844E2C`: another random, turned right 16, plus the clock, `and`
//! `dword_844E34[settings +0x2748]`, not 0); while they move, the shape index
//! (`dword_84460C`) from a running count (`dword_82E38A` plus `g_marksFrameTicks` by
//! `flt_89C9A0`, or by `flt_89C99C` when the settings' +0x274C is 1 and the index >> 3 is
//! not 0), its bits 16 .. 23, and `pick_shapes` again.
//!
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::shapes_frame`. Refuses null settings, a +0x2748 of 9 or more when it is read
//! (`dword_844E34` holds 9), and its steps' refusals; on a refusal nothing stays written
//! (all it may write is put back).

use crate::effects::emitters::next_c5;
use crate::effects::mask_store::Masks;
use crate::effects::pick_shapes::pick_shapes;
use crate::effects::shape_points::turn_all_shapes;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::race::owner_shared::Shared;

use crate::math::x87;
const MASK_COUNT: u32 = 9;

pub fn run_shapes(
    shared: &Shared,
    masks: &Masks,
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    let marks = crate::effects::setup::marks()?;
    {
        pick_shapes(masks, drops)?;
        turn_all_shapes(masks, drops, effects)?;
        let clock: u32 = shared.physics_tables.scene_clock.bits();
        if clock as i32 >= drops.picks.next.get() as i32 {
            let r = next_c5(&setup.mask_constants.time_random);
            drops
                .picks
                .next
                .set((r & 0xFF).wrapping_add(shared.physics_tables.scene_clock.bits()));
            let mask = marks.listener_switch_7.get();
            if mask >= MASK_COUNT {
                return Err(BadInput("a mask number past dword_844E34's 9").into());
            }
            let moving = next_c5(&setup.mask_constants.move_random)
                .rotate_right(16)
                .wrapping_add(clock)
                & drops.picks.masks[mask as usize].get();
            drops.picks.moving.set(1);
            if moving == 0 {
                drops.picks.moving.set(0);
                return Ok(());
            }
        }
        if drops.picks.moving.get() == 0 {
            return Ok(());
        }
        let rate = if marks.listener_switch_5.get() == 1 {
            (drops.picks.index.get() >> 3 != 0).then_some(&setup.mask_constants.rates[0])
        } else {
            Some(&setup.mask_constants.rates[1])
        };
        if let Some(rate) = rate {
            let count = x87::int_times_plus_int_to_int(
                setup.clock.frame_ticks.get() as i32,
                rate.bits(),
                masks.vars.count.get() as i32,
            );
            masks.vars.count.set(count as u32);
            drops.picks.index.set((count as u32 >> 16) & 0xFF);
        }
        pick_shapes(masks, drops)?;
    }
    Ok(())
}
