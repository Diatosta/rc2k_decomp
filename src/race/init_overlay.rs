//! `race::init_overlay` (0x45A9DE): the overlay's stage parts (`initoverlay`).
//!
//! The parts are the stretches between the checkpoints `race::init_checkpoints` listed
//! (`checkpoints`, `split_colour_count` of them): each one's length, the difference of its two
//! checkpoints' `g_checkpointDistances` halved (`shr`, so a negative one comes out huge) /
//! 65536 / 1609.344 (metres in a mile) and doubled, into `g_overlayPartMiles` (floats, 0x40),
//! their sum into `g_overlayMiles`, `g_overlayParts` their count, the two dwords of each
//! part at `overlay_part_kinds_a` and `overlay_part_kinds_b` 0. With fewer than two checkpoint kinds or no
//! parts, one part of 1.0. Then six sizes (0x40, 0x80, 0x50, twice), each car block's
//! +0xF78 .. +0xF98 and +0xFA0 0, `distance` and `flt_6D146E` 0. The x87 steps through
//! `math::x87`; `scratch` left as the original's `fild` leaves it.
//!
//! One caller, `race::loadlevel_init` ("loadlevel_init : initoverlay"; no `pusha`). Refuses more
//! than 0x40 parts (the original writes past its three lists).

use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner::Race;

const ONE: u32 = 0x3F80_0000;

/// `race::init_overlay` (see the header).
pub fn init_overlay(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::panel::{overlay_parts, half_length, OVERLAY_SIZES,
        // OVERLAY_CLEARED}`; the miles the x87's.
        use rc2k_core::race::panel::{OVERLAY_CLEARED, OVERLAY_SIZES, half_length, overlay_parts};
        let (found, parts) = overlay_parts(race.shared.vars.checkpoint_count.get(), || {
            race.physics.split_tables().split_colour_count.get()
        })?;
        if let Some(count) = found {
            race.frame.panel_and_overlay.panel.overlay_parts.set(count);
        }
        match parts {
            Some(count) => {
                race.frame.panel_and_overlay.panel.overlay_miles.set(0);
                let (scale, mile) = (
                    race.course.numbers.k65536.get(),
                    race.course.numbers.metres_in_mile.get(),
                );
                // The list's entries 0 .. count (at most 0x40, `overlay_parts`) are the ones
                // `race::init_checkpoints` wrote (`split_colour_count` is their count - 1),
                // each a checkpoint number below `last_checkpoint`, inside the distances.
                let list = &race.physics.split_tables().checkpoints;
                let distances = &race.checkpoints().distances;
                let distance = |k: usize| -> Result<u32, crate::errors::BadInput> {
                    let n = list
                        .get(k)
                        .ok_or(crate::errors::BadInput(
                            "an overlay part past the split list",
                        ))?
                        .get();
                    Ok(distances
                        .get(n as usize)
                        .ok_or(crate::errors::BadInput(
                            "a split's checkpoint past the distances",
                        ))?
                        .get())
                };
                for n in 0..count {
                    let end = distance(n as usize + 1)?;
                    let start = distance(n as usize)?;
                    let half = half_length(end, start);
                    scene.drawing.scene.scratch.set(half);
                    let miles = x87::int_div_div_doubled(half as i32, scale, mile);
                    race.frame.panel_and_overlay.overlay.overlay_part_miles[n as usize].set(miles); // g_overlayPartMiles
                    race.frame.panel_and_overlay.panel.overlay_miles.set(
                        x87::add(
                            miles,
                            race.frame.panel_and_overlay.panel.overlay_miles.get(),
                        )
                        .to_bits(),
                    );
                    race.frame.panel_and_overlay.overlay.overlay_part_kinds_a[n as usize].set(0); // dword_6D160C
                    race.frame.panel_and_overlay.overlay.overlay_part_kinds_b[n as usize].set(0); // dword_6D170C
                }
            }
            None => {
                race.frame.panel_and_overlay.panel.overlay_parts.set(1);
                race.frame.panel_and_overlay.overlay.overlay_part_miles[0].set(1.0);
                race.frame.panel_and_overlay.panel.overlay_miles.set(ONE);
                race.frame.panel_and_overlay.overlay.overlay_part_kinds_a[0].set(0);
                race.frame.panel_and_overlay.overlay.overlay_part_kinds_b[0].set(0);
            }
        }
        // The sizes (`rc2k_core::race::panel::OVERLAY_SIZES`): dword_6D1486 .. dword_6D149E.
        let sizes = [
            &race.frame.panel_and_overlay.panel.lights[0].width,
            &race.frame.panel_and_overlay.panel.lights[0].alpha,
            &race.frame.panel_and_overlay.panel.lights[0].drop,
            &race.frame.panel_and_overlay.panel.lights[1].width,
            &race.frame.panel_and_overlay.panel.lights[1].alpha,
            &race.frame.panel_and_overlay.panel.lights[1].drop,
        ];
        for (cell, value) in sizes.into_iter().zip(OVERLAY_SIZES) {
            cell.set(value);
        }
        for car in 0..race.shared.work.car_blocks.len() {
            for off in OVERLAY_CLEARED {
                crate::race::race_init::state_cell(race.shared.race_car(car)?, off)?.set(0);
            }
        }
        for distance in &race.frame.panel_and_overlay.panel.distance {
            distance.set(0);
        }
    }
    Ok(())
}
