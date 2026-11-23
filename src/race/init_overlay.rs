//! `race::init_overlay`: sets up the overlay's stage parts.
//!
//! The parts are the stretches between the checkpoints `race::init_checkpoints` listed
//! (`checkpoints`, `split_colour_count` of them): each one's length is the difference of its two
//! checkpoints' `checkpoint_distances`, halved with a logical shift (so a negative difference comes
//! out huge), / 65536 / 1609.344 (metres in a mile) and doubled. The lengths go into
//! `overlay_part_miles` (floats, at most 0x40), their sum into `overlay_miles`, their count into
//! `overlay_parts`, and the two part-kind dwords of each part are set to 0. With fewer than two
//! checkpoint kinds or no parts, there is one part of 1.0. Then six sizes (0x40, 0x80, 0x50, twice)
//! are set and each car block's event codes, event count and event position are cleared. The x87
//! steps go through `math::x87`; `scratch` is left as the `fild` leaves it.
//!
//! Refuses more than 0x40 parts.

use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner::Race;

const ONE: u32 = 0x3F80_0000;

/// Sets up the overlay's stage parts (see the module header).
pub(super) fn init_overlay(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    crate::hud::layout_common::start_gear_log(&mut race.hud.get().borrow_mut());
    race.frame.panel.borrow_mut().overview_drawn = 0;
    {
        // Logic in `rc2k_core::race::panel::{overlay_parts, half_length, OVERLAY_SIZES,
        // OVERLAY_CLEARED}`; the miles the x87's.
        use rc2k_core::race::panel::{OVERLAY_CLEARED, OVERLAY_SIZES, half_length, overlay_parts};
        let (found, parts) = overlay_parts(race.shared.vars.get().checkpoint_count.get(), || {
            race.physics.split_tables().split_colour_count
        })?;
        if let Some(count) = found {
            race.frame.panel.borrow_mut().overlay_parts = count;
        }
        match parts {
            Some(count) => {
                race.frame.panel.borrow_mut().overlay_miles = 0;
                let (scale, mile) = (
                    race.course.numbers.k65536.get(),
                    race.course.numbers.metres_in_mile.get(),
                );
                // The list's entries 0 .. count (at most 0x40) were written by
                // `race::init_checkpoints` (`split_colour_count` is their count - 1), each a
                // checkpoint number below `last_checkpoint`, inside the distances.
                let list = race.physics.split_tables().checkpoints;
                let checkpoints = race.checkpoints();
                let distance = |k: usize| -> Result<u32, crate::errors::BadInput> {
                    let n = list.get(k).ok_or(crate::errors::BadInput(
                        "an overlay part past the split list",
                    ))?;
                    checkpoints
                        .read(|tables| tables.distances.get(*n as usize).copied())
                        .ok_or(crate::errors::BadInput(
                            "a split's checkpoint past the distances",
                        ))
                };
                for n in 0..count {
                    let end = distance(n as usize + 1)?;
                    let start = distance(n as usize)?;
                    let half = half_length(end, start);
                    scene.draw_work_mut().scratch = half;
                    let miles = x87::int_div_div_doubled(half as i32, scale, mile);
                    race.frame.overlay_mut().overlay_part_miles[n as usize] = miles;
                    let value = x87::add(miles, race.frame.panel.borrow().overlay_miles).to_bits();
                    race.frame.panel.borrow_mut().overlay_miles = value;
                    race.frame.overlay_mut().overlay_part_kinds_a[n as usize] = 0;
                    race.frame.overlay_mut().overlay_part_kinds_b[n as usize] = 0;
                }
            }
            None => {
                race.frame.panel.borrow_mut().overlay_parts = 1;
                race.frame.overlay_mut().overlay_part_miles[0] = 1.0;
                race.frame.panel.borrow_mut().overlay_miles = ONE;
                race.frame.overlay_mut().overlay_part_kinds_a[0] = 0;
                race.frame.overlay_mut().overlay_part_kinds_b[0] = 0;
            }
        }
        // The sizes (`rc2k_core::race::panel::OVERLAY_SIZES`).
        {
            let mut panel = race.frame.panel.borrow_mut();
            let [first, second] = &mut panel.lights;
            let sizes = [
                &mut first.width,
                &mut first.alpha,
                &mut first.drop,
                &mut second.width,
                &mut second.alpha,
                &mut second.drop,
            ];
            for (cell, value) in sizes.into_iter().zip(OVERLAY_SIZES) {
                *cell = value;
            }
        }
        for car in 0..race.shared.work().car_blocks.len() {
            for word in OVERLAY_CLEARED {
                *crate::race::race_init::block_word(&mut *race.shared.race_car_mut(car)?, word)? =
                    0;
            }
        }
        race.frame.panel.borrow_mut().distance = [0; 2];
        race.frame.panel.borrow_mut().network_cars = 0;
    }
    Ok(())
}
