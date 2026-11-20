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
    race.frame
        .panel_and_overlay
        .get()
        .panel
        .overview_drawn
        .set(0);
    {
        // Logic in `rc2k_core::race::panel::{overlay_parts, half_length, OVERLAY_SIZES,
        // OVERLAY_CLEARED}`; the miles the x87's.
        use rc2k_core::race::panel::{OVERLAY_CLEARED, OVERLAY_SIZES, half_length, overlay_parts};
        let (found, parts) = overlay_parts(race.shared.vars.get().checkpoint_count.get(), || {
            race.physics.split_tables().split_colour_count
        })?;
        if let Some(count) = found {
            race.frame
                .panel_and_overlay
                .get()
                .panel
                .overlay_parts
                .set(count);
        }
        match parts {
            Some(count) => {
                race.frame
                    .panel_and_overlay
                    .get()
                    .panel
                    .overlay_miles
                    .set(0);
                let (scale, mile) = (
                    race.course.numbers.get().k65536.get(),
                    race.course.numbers.get().metres_in_mile.get(),
                );
                // The list's entries 0 .. count (at most 0x40) were written by
                // `race::init_checkpoints` (`split_colour_count` is their count - 1), each a
                // checkpoint number below `last_checkpoint`, inside the distances.
                let list = race.physics.split_tables().checkpoints;
                let distances = &race.checkpoints().distances;
                let distance = |k: usize| -> Result<u32, crate::errors::BadInput> {
                    let n = list.get(k).ok_or(crate::errors::BadInput(
                        "an overlay part past the split list",
                    ))?;
                    Ok(distances
                        .get(*n as usize)
                        .ok_or(crate::errors::BadInput(
                            "a split's checkpoint past the distances",
                        ))?
                        .get())
                };
                for n in 0..count {
                    let end = distance(n as usize + 1)?;
                    let start = distance(n as usize)?;
                    let half = half_length(end, start);
                    scene.drawing.scene.get().scratch.set(half);
                    let miles = x87::int_div_div_doubled(half as i32, scale, mile);
                    race.frame
                        .panel_and_overlay
                        .get()
                        .overlay
                        .overlay_part_miles[n as usize]
                        .set(miles);
                    race.frame.panel_and_overlay.get().panel.overlay_miles.set(
                        x87::add(
                            miles,
                            race.frame.panel_and_overlay.get().panel.overlay_miles.get(),
                        )
                        .to_bits(),
                    );
                    race.frame
                        .panel_and_overlay
                        .get()
                        .overlay
                        .overlay_part_kinds_a[n as usize]
                        .set(0);
                    race.frame
                        .panel_and_overlay
                        .get()
                        .overlay
                        .overlay_part_kinds_b[n as usize]
                        .set(0);
                }
            }
            None => {
                race.frame
                    .panel_and_overlay
                    .get()
                    .panel
                    .overlay_parts
                    .set(1);
                race.frame
                    .panel_and_overlay
                    .get()
                    .overlay
                    .overlay_part_miles[0]
                    .set(1.0);
                race.frame
                    .panel_and_overlay
                    .get()
                    .panel
                    .overlay_miles
                    .set(ONE);
                race.frame
                    .panel_and_overlay
                    .get()
                    .overlay
                    .overlay_part_kinds_a[0]
                    .set(0);
                race.frame
                    .panel_and_overlay
                    .get()
                    .overlay
                    .overlay_part_kinds_b[0]
                    .set(0);
            }
        }
        // The sizes (`rc2k_core::race::panel::OVERLAY_SIZES`).
        let sizes = [
            &race.frame.panel_and_overlay.get().panel.lights[0].width,
            &race.frame.panel_and_overlay.get().panel.lights[0].alpha,
            &race.frame.panel_and_overlay.get().panel.lights[0].drop,
            &race.frame.panel_and_overlay.get().panel.lights[1].width,
            &race.frame.panel_and_overlay.get().panel.lights[1].alpha,
            &race.frame.panel_and_overlay.get().panel.lights[1].drop,
        ];
        for (cell, value) in sizes.into_iter().zip(OVERLAY_SIZES) {
            cell.set(value);
        }
        for car in 0..race.shared.work.get().car_blocks.len() {
            for word in OVERLAY_CLEARED {
                crate::race::race_init::block_word(race.shared.race_car(car)?, word)?.set(0);
            }
        }
        for distance in &race.frame.panel_and_overlay.get().panel.distance {
            distance.set(0);
        }
        race.frame.panel_and_overlay.get().panel.network_cars.set(0);
    }
    Ok(())
}
