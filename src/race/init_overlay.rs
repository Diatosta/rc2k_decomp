//! `race::init_overlay` (0x45A9DE): the overlay's stage parts (`initoverlay`).
//!
//! In:  nothing
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24)
//!
//! The parts are the stretches between the checkpoints `race::init_checkpoints` listed
//! (`dword_60102C`, `dword_601830` of them): each one's length, the difference of its two
//! checkpoints' `g_checkpointDistances` halved (`shr`, so a negative one comes out huge) /
//! 65536 / 1609.344 (metres in a mile) and doubled, into `g_overlayPartMiles` (floats, 0x40),
//! their sum into `g_overlayMiles`, `g_overlayParts` their count, the two dwords of each
//! part at `dword_6D160C` and `dword_6D170C` 0. With fewer than two checkpoint kinds or no
//! parts, one part of 1.0. Then six sizes (0x40, 0x80, 0x50, twice), each car block's
//! +0xF78 .. +0xF98 and +0xFA0 0, `flt_6D146A` and `flt_6D146E` 0. The x87 steps through
//! `math::x87`; `dword_62400C` left as the original's `fild` leaves it.
//!
//! One caller, `sub_44426F` ("loadlevel_init : initoverlay"; no `pusha`). Refuses more than
//! 0x40 parts (the original writes past its three lists).

use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner::Race;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;

const ONE: u32 = 0x3F80_0000;

/// `race::init_overlay` (see the header).
pub fn init_overlay(race: &Race) -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::panel::{overlay_parts, half_length, OVERLAY_SIZES,
        // OVERLAY_CLEARED}`; the miles the x87's.
        use rc2k_core::race::panel::{OVERLAY_CLEARED, OVERLAY_SIZES, half_length, overlay_parts};
        let (found, parts) = overlay_parts(RACE_VARS.checkpoint_count.get(), || {
            race.physics.split_tables.split_colour_count.get()
        })?;
        if let Some(count) = found {
            race.frame.panel_and_overlay.panel.overlay_parts.set(count);
        }
        match parts {
            Some(count) => {
                race.frame.panel_and_overlay.panel.overlay_miles.set(0);
                let (scale, mile): (u32, u32) = (
                    race.course.numbers.k65536.bits(),
                    race.course.numbers.metres_in_mile.bits(),
                );
                // By address: the list runs past its 0x101 entries, and a face number past
                // the distances is the original's read.
                let list = &race.physics.split_tables.checkpoints;
                let distances = &crate::race::checkpoints_store::CHECKPOINTS.distances;
                let distance = |k: u32| {
                    crate::session::read_on_in(
                        &crate::race::buffers_store::BUFFERS,
                        distances,
                        k as usize,
                        "an overlay variable",
                    )
                };
                for n in 0..count {
                    let end = distance(crate::session::read_on_in(
                        &race.physics.split_tables,
                        list,
                        n as usize + 1,
                        "an overlay variable",
                    )?)?;
                    let start = distance(crate::session::read_on_in(
                        &race.physics.split_tables,
                        list,
                        n as usize,
                        "an overlay variable",
                    )?)?;
                    let half = half_length(end, start);
                    crate::scene::owner::scene().drawing.scene.scratch.set(half);
                    let miles = x87::int_div_div_doubled(half as i32, scale, mile);
                    race.frame.panel_and_overlay.overlay.overlay_part_miles[n as usize]
                        .set_bits(miles); // g_overlayPartMiles
                    race.frame
                        .panel_and_overlay
                        .panel
                        .overlay_miles
                        .set(x87::add(
                            miles,
                            race.frame.panel_and_overlay.panel.overlay_miles.get(),
                        ));
                    race.frame.panel_and_overlay.overlay.overlay_part_kinds_a[n as usize].set(0); // dword_6D160C
                    race.frame.panel_and_overlay.overlay.overlay_part_kinds_b[n as usize].set(0); // dword_6D170C
                }
            }
            None => {
                race.frame.panel_and_overlay.panel.overlay_parts.set(1);
                race.frame.panel_and_overlay.overlay.overlay_part_miles[0].set_bits(ONE);
                race.frame.panel_and_overlay.panel.overlay_miles.set(ONE);
                race.frame.panel_and_overlay.overlay.overlay_part_kinds_a[0].set(0);
                race.frame.panel_and_overlay.overlay.overlay_part_kinds_b[0].set(0);
            }
        }
        // The sizes (`rc2k_core::race::panel::OVERLAY_SIZES`): dword_6D1486 .. dword_6D149E.
        let sizes = [
            &race.frame.panel_and_overlay.panel.lights[0].fade,
            &race.frame.panel_and_overlay.panel.lights[0].size,
            &race.frame.panel_and_overlay.panel.lights[0].shift,
            &race.frame.panel_and_overlay.panel.lights[1].fade,
            &race.frame.panel_and_overlay.panel.lights[1].size,
            &race.frame.panel_and_overlay.panel.lights[1].shift,
        ];
        for (cell, value) in sizes.into_iter().zip(OVERLAY_SIZES) {
            cell.set(value);
        }
        for car in 0..RACE_WORK.car_blocks.len() {
            let block = RACE_WORK.car_block(car);
            for off in OVERLAY_CLEARED {
                crate::race::race_init::state_cell(crate::race::trap::car_at(block)?, off)?.set(0);
            }
        }
        for distance in &race.frame.panel_and_overlay.panel.distance {
            distance.set(0);
        }
    }
    Ok(())
}
