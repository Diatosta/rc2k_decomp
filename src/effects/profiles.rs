//! `effects::make_profiles`, the body of `init_marks`' step 3: two rings of 0x100 indexes, once the
//! 13 profile tables (0x100 whole numbers and their 0x100 differences each), then the corners from
//! the course and `allocate_links`, `reset_settings_blocks`.

use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::PlainDword;

pub const COURSE_CORNERS: u32 = 0x1C_2000;
pub const CORNERS_LEN: usize = 0x8000;

/// The tables of the three ramps, in `ProfileTables::tables`.
const RAMPS: [usize; 3] = [0, 3, 4];
/// The tables of the eight sines.
const SINES: [usize; 8] = [5, 6, 7, 8, 9, 10, 11, 12];
/// The tables of the two curves.
const CURVES: [usize; 2] = [1, 2];

/// The profiles' work, `DrawWork.profile_work[0]` .. `DrawWork.profile_work[10]`.
fn work(n: usize, v: f32, effects: &crate::effects::owner::Effects) {
    effects.frame.draw_work.profile_work[n].set(v);
}

/// A table's second half: each the next less this (whole numbers, so the last reads the
/// second half's first, already a float), the first half made floats.
fn differences(table: &ProfileTable) {
    for n in 0..0x100 {
        table[0x100 + n].set(bh::st(
            bh::int(table[n + 1].get() as i32) - bh::int(table[n].get() as i32),
        ));
        table[n].set(x87::from_int(table[n].get() as i32).to_bits());
    }
}

/// The once-only tables.
fn make_tables(paths: &Paths, effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    let frame = &effects.frame;
    {
        for &table in &RAMPS {
            let t = &paths.profile_tables.tables[table];
            let step = bh::stf(
                (bh::int(t[1].get() as i32) - bh::int(t[0].get() as i32))
                    / bh::ld(frame.draw_frame.profile_steps[0].get()),
            );
            work(2, step, effects);
            let mut v = x87::from_int(t[0].get() as i32);
            work(0, v, effects);
            for cell in t.iter().take(0x100) {
                cell.set(x87::to_int(v) as u32);
                v = x87::add(v, step);
                work(0, v, effects);
            }
            differences(t);
        }
        for &table in &SINES {
            let t = &paths.profile_tables.tables[table];
            let p: [f32; 4] = std::array::from_fn(|i| x87::loaded(t[i].get()));
            for (i, &v) in p.iter().enumerate() {
                work(3 + i, v, effects);
            }
            let mut a = p[0];
            for cell in t.iter().take(0x100) {
                cell.set(bh::to_int(bh::ld(a).sin() * bh::ld(p[2]) + bh::ld(p[1])) as u32);
                a = x87::add(a, p[3]);
                work(3, a, effects);
            }
            differences(t);
        }
        for &table in &CURVES {
            let t = &paths.profile_tables.tables[table];
            let mut x = x87::loaded(t[0].get());
            work(7, x, effects);
            let step = x87::sub_div(t[1].get(), t[0].get(), &frame.draw_frame.profile_steps[1]);
            work(8, step, effects);
            let (scale, top) = (x87::loaded(t[2].get()), x87::loaded(t[3].get()));
            work(9, scale, effects);
            work(10, top, effects);
            for cell in t.iter().take(0x100) {
                cell.set(x87::negated_square_times_plus_to_int(x, scale, top) as u32);
                x = x87::add(x, step);
                work(7, x, effects);
            }
            differences(t);
        }
    }
    Ok(())
}

/// `effects::make_profiles`: `ProfileRings.zero` 0, the rings' before and after (of 0x100);
/// unless done before: the 3 ramps (start to end over 256 steps), 8
/// sines (base + amplitude sin, the angle stepped), 2 curves (top - x² scale), each table's
/// differences after it (a fourth list is of no tables), the done flag set; then
/// 0x8000 bytes of corners from the course info + 0x1C2000 (twice), `allocate_links` and
/// `reset_settings_blocks`.
pub fn make_profiles(
    paths: &Paths,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        paths.profile_rings.zero.set(0);
        for n in 0..0x100u32 {
            paths.profile_rings.before[n as usize].set(n.wrapping_sub(1) & 0xFF);
            paths.profile_rings.after[n as usize].set((n + 1) & 0xFF);
        }
        if !effects.frame.draw_frame.profiles_done.is_set() {
            make_tables(paths, effects)?;
            effects.frame.draw_frame.profiles_done.set(true);
        }
        if !crate::race::course_places::info_held(&effects.stage()?.course) {
            return Err(BadInput("no course info").into());
        }
        let corners = crate::race::course_places::info_bytes(
            &effects.stage()?.course,
            COURSE_CORNERS,
            CORNERS_LEN,
        )?;
        crate::memory::cells::Store::store(&effects.blocks.corners, 0, &corners)?;
        crate::effects::blocks::allocate_links(scene, effects)?;
        crate::effects::blocks::reset_settings_blocks(effects)?;
    }
    Ok(())
}

/// The profiles' rings of indexes.
#[derive(rc2k_derive::Cells)]
pub struct ProfileRings {
    /// Each of the 0x100 indexes' one before it (`effects::make_profiles`).
    pub before: [PlainDword; 256],
    /// Each index's one after it.
    pub after: [PlainDword; 256],
    /// 0 (`effects::make_profiles` writes it).
    pub zero: PlainDword,
}

impl ProfileRings {
    /// The values the game starts with.
    pub const fn new() -> Self {
        ProfileRings {
            before: [const { PlainDword::new(0x0) }; 256],
            after: [const { PlainDword::new(0x0) }; 256],
            zero: PlainDword::new(0x0),
        }
    }
}

/// One profile table: 0x100 whole numbers and then their 0x100 differences (the first
/// dwords hold the table's inputs until `effects::make_profiles` makes it).
pub type ProfileTable = [PlainDword; 0x200];

/// A profile table with the given first values (index, value).
const fn table(first: &[(usize, u32)]) -> ProfileTable {
    let mut t = [const { PlainDword::new(0) }; 0x200];
    let mut k = 0;
    while k < first.len() {
        t[first[k].0] = PlainDword::new(first[k].1);
        k += 1;
    }
    t
}

/// The profile tables (13 of 0x800 bytes, in this order of memory: ramp 0, curves 0 and 1, ramps 1
/// and 2, sines 0 .. 7).
#[derive(rc2k_derive::Cells)]
pub struct ProfileTables {
    /// Ramp 0, curve 0, curve 1, ramp 1, ramp 2, then sines 0 .. 7 (`RAMPS`, `CURVES`, `SINES`).
    pub tables: [ProfileTable; 13],
}

impl ProfileTables {
    /// The values the game starts with.
    pub const fn new() -> Self {
        ProfileTables {
            tables: [
                table(&[]),
                table(&[(0, 0xC0000000), (2, 0x469C4000), (3, 0x48435000)]),
                table(&[(1, 0x3F000000), (2, 0xC87DE800)]),
                table(&[(0, 0xFF)]),
                table(&[(0, 0x7F)]),
                table(&[(2, 0x466A6000), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48212200), (3, 0x3BC90FDB)]),
                table(&[(2, 0x4851F600), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48816500), (3, 0x3BC90FDB)]),
                table(&[(2, 0x4899CF00), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48B23900), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48CAA300), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48E30D00), (3, 0x3BC90FDB)]),
            ],
        }
    }
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
