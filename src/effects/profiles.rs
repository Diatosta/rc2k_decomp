//! `effects::make_profiles`, the body of `init_marks`' step 3: two rings of 0x100 indexes, once the
//! 13 profile tables (0x100 whole numbers and their 0x100 differences each), then the corners from
//! the course and `allocate_links`, `reset_settings_blocks`.

use crate::effects::profiles_store::{ProfileTable, first_tables};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;

const COURSE_CORNERS: u32 = 0x1C_2000;
pub(super) const CORNERS_LEN: usize = 0x8000;

/// The tables of the three ramps, in `first_tables`.
const RAMPS: [usize; 3] = [0, 3, 4];
/// The tables of the eight sines.
const SINES: [usize; 8] = [5, 6, 7, 8, 9, 10, 11, 12];
/// The tables of the two curves.
const CURVES: [usize; 2] = [1, 2];

/// The profiles' work, `DrawWork.profile_work[0]` .. `DrawWork.profile_work[10]`.
fn work(n: usize, v: f32, effects: &crate::effects::owner::Effects) {
    effects.frame.draw_work.borrow_mut().profile_work[n] = v;
}

/// A table's second half: each the next less this (whole numbers, so the last reads the
/// second half's first, already a float), the first half made floats.
fn differences(table: &mut ProfileTable) {
    for n in 0..0x100 {
        table[0x100 + n] = bh::st(bh::int(table[n + 1] as i32) - bh::int(table[n] as i32));
        table[n] = x87::from_int(table[n] as i32).to_bits();
    }
}

/// The once-only tables.
fn make_tables(effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    let profile_steps = effects.frame.draw_frame.borrow().profile_steps;
    let mut tables = first_tables();
    {
        for &table in &RAMPS {
            let t = &mut tables[table];
            let step =
                bh::stf((bh::int(t[1] as i32) - bh::int(t[0] as i32)) / bh::ld(profile_steps[0]));
            work(2, step, effects);
            let mut v = x87::from_int(t[0] as i32);
            work(0, v, effects);
            for cell in t.iter_mut().take(0x100) {
                *cell = x87::to_int(v) as u32;
                v = x87::add(v, step);
                work(0, v, effects);
            }
            differences(t);
        }
        for &table in &SINES {
            let t = &mut tables[table];
            let p: [f32; 4] = std::array::from_fn(|i| x87::loaded(t[i]));
            for (i, &v) in p.iter().enumerate() {
                work(3 + i, v, effects);
            }
            let mut a = p[0];
            for cell in t.iter_mut().take(0x100) {
                *cell = bh::to_int(bh::ld(a).sin() * bh::ld(p[2]) + bh::ld(p[1])) as u32;
                a = x87::add(a, p[3]);
                work(3, a, effects);
            }
            differences(t);
        }
        for &table in &CURVES {
            let t = &mut tables[table];
            let mut x = x87::loaded(t[0]);
            work(7, x, effects);
            let step = x87::sub_div(t[1], t[0], profile_steps[1]);
            work(8, step, effects);
            let (scale, top) = (x87::loaded(t[2]), x87::loaded(t[3]));
            work(9, scale, effects);
            work(10, top, effects);
            for cell in t.iter_mut().take(0x100) {
                *cell = x87::negated_square_times_plus_to_int(x, scale, top) as u32;
                x = x87::add(x, step);
                work(7, x, effects);
            }
            differences(t);
        }
    }
    Ok(())
}

/// `effects::make_profiles`: unless done before: the 3 ramps (start to end over 256 steps), 8
/// sines (base + amplitude sin, the angle stepped), 2 curves (top - x² scale), each table's
/// differences after it (a fourth list is of no tables), the done flag set; then
/// 0x8000 bytes of corners from the course info + 0x1C2000 (twice), `allocate_links` and
/// `reset_settings_blocks`.
pub(super) fn make_profiles(
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        if !effects.frame.draw_frame.borrow().profiles_done {
            make_tables(effects)?;
            effects.frame.draw_frame.borrow_mut().profiles_done = true;
        }
        if !crate::race::course_places::info_held(&effects.stage()?.course) {
            return Err(BadInput("no course info").into());
        }
        let corners = crate::race::course_places::info_bytes(
            &effects.stage()?.course,
            COURSE_CORNERS,
            CORNERS_LEN,
        )?;
        effects.blocks.corners.set_from_bytes(&corners)?;
        crate::effects::blocks::allocate_links(scene, effects)?;
        crate::effects::blocks::reset_settings_blocks(effects)?;
    }
    Ok(())
}

#[cfg(test)]
mod typed_tests {
    use super::*;

    /// The tables the image's pointer lists named, by address.
    #[test]
    fn table_numbers_name_the_tables_the_addresses_did() {
        let named = |ida: usize| (ida - 0x8E506D) / 0x800;
        assert_eq!([0x8E506D, 0x8E686D, 0x8E706D].map(named), RAMPS);
        assert_eq!(
            [
                0x8E786D, 0x8E806D, 0x8E886D, 0x8E906D, 0x8E986D, 0x8EA06D, 0x8EA86D, 0x8EB06D
            ]
            .map(named),
            SINES
        );
        assert_eq!([0x8E586D, 0x8E606D].map(named), CURVES);
    }
}
