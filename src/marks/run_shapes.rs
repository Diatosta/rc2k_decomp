//! `marks::run_shapes` (0x4BDF72): the mask shapes' frame: `marks::pick_shapes`,
//! `marks::turn_all_shapes`; then when `g_sceneClock` reaches `dword_844E30`, the next
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

use crate::errors::{BadInput, Refused};
use crate::marks::emitters::next_c5;
use crate::marks::pick_shapes::pick_shapes;
use crate::marks::setup::CLOCK;
use crate::marks::shape_points::turn_all_shapes;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::physics::tables_store::PHYSICS;
const NEXT_RVA: Slot = Slot::at(0x844E30); // dword_844E30
const TIME_RANDOM_RVA: Slot = Slot::at(0x89C994); // dword_89C994
const MOVE_RANDOM_RVA: Slot = Slot::at(0x89C998); // dword_89C998
const MASKS_RVA: Slot = Slot::at(0x844E34); // dword_844E34: 9 masks
const MASKS: u32 = 9;
const MOVING_RVA: Slot = Slot::at(0x844E2C); // dword_844E2C
const INDEX_RVA: Slot = Slot::at(0x84460C); // dword_84460C
const COUNT_RVA: Slot = Slot::at(0x82E38A); // dword_82E38A
const RATE_RVA: [Slot; 2] = [Slot::at(0x89C99C), Slot::at(0x89C9A0)];

/// The Rust version (see the header).
pub unsafe fn run_shapes() -> Result<(), Refused> {
    let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let get = |o: u32| unsafe { ((settings + o) as *const u32).read_unaligned() };
    unsafe {
        pick_shapes()?;
        turn_all_shapes()?;
        let clock: u32 = PHYSICS.scene_clock.bits();
        if clock as i32 >= game_read::<i32>(NEXT_RVA) {
            let r = next_c5(TIME_RANDOM_RVA);
            game_write(
                NEXT_RVA,
                (r & 0xFF).wrapping_add(PHYSICS.scene_clock.bits()),
            );
            let mask = get(0x2748);
            if mask >= MASKS {
                return Err(BadInput("a mask number past dword_844E34's 9").into());
            }
            let moving = next_c5(MOVE_RANDOM_RVA)
                .rotate_right(16)
                .wrapping_add(clock)
                & game_read::<u32>(MASKS_RVA + 4 * mask as usize);
            game_write(MOVING_RVA, 1u32);
            if moving == 0 {
                game_write(MOVING_RVA, 0u32);
                return Ok(());
            }
        }
        if game_read::<u32>(MOVING_RVA) == 0 {
            return Ok(());
        }
        let rate = if get(0x274C) == 1 {
            (game_read::<u32>(INDEX_RVA) >> 3 != 0).then_some(RATE_RVA[0])
        } else {
            Some(RATE_RVA[1])
        };
        if let Some(rate) = rate {
            let count = x87::int_times_plus_int_to_int(
                CLOCK.frame_ticks.get() as i32,
                game_read(rate),
                game_read(COUNT_RVA),
            );
            game_write(COUNT_RVA, count);
            game_write(INDEX_RVA, (count as u32 >> 16) & 0xFF);
        }
        pick_shapes()?;
    }
    Ok(())
}
