//! `effects::make_profiles` (0x4C3B01), the body of `init_marks`' step 3 (0x4C4E1C is
//! only its call): two rings of 0x100 indexes, once the 13 profile
//! tables (0x100 whole numbers and their 0x100 differences each), then the corners from
//! the course and `allocate_links`, `reset_settings_blocks`.

use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner_course::Course;
use crate::session::PlainDword;

pub const RESUME: u32 = 0x4C3EA6; // loc_4C3EA6
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

/// `effects::make_profiles` (0x4C3B01): `ProfileRings.zero` 0, the rings' before and after (of
/// 0x100); unless done (`dword_91FF3E` = `resume`): the 3 ramps (start to end over 256
/// steps), 8 sines (base + amplitude sin, the angle stepped), 2 curves (top - x² scale),
/// each table's differences after it (a fourth list is of no tables), the mark `resume`;
/// then 0x8000 bytes of corners from `g_courseInfo` + 0x1C2000 (twice, as the original),
/// `allocate_links` and `reset_settings_blocks`. Refuses a mark that is neither 0 nor
/// `resume` (the original jumps to it). `resume` is `loc_4C3EA6`.
pub fn make_profiles(
    course: &Course,
    paths: &Paths,
    resume: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let done: u32 = effects.frame.draw_frame.profiles_done.get();
        if done != 0 && done != resume {
            return Err(BadInput("the profiles' mark is neither 0 nor loc_4C3EA6").into());
        }
        paths.profile_rings.zero.set(0);
        for n in 0..0x100u32 {
            paths.profile_rings.before[n as usize].set(n.wrapping_sub(1) & 0xFF);
            paths.profile_rings.after[n as usize].set((n + 1) & 0xFF);
        }
        if done == 0 {
            make_tables(paths, effects)?;
            effects.frame.draw_frame.profiles_done.set(resume);
        }
        if course.lists.get().course_info.get() == 0 {
            return Err(BadInput("no course info (g_courseInfo 0)").into());
        }
        let corners = crate::race::course_places::info_bytes(COURSE_CORNERS, CORNERS_LEN)?;
        crate::cells::Store::store(&effects.blocks.corners, 0, &corners)?;
        crate::effects::blocks::allocate_links(effects)?;
        crate::effects::blocks::reset_settings_blocks(effects)?;
    }
    Ok(())
}

/// The profiles' rings of indexes (IDA 0x8DD061 .. 0x8DD865).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ProfileRings {
    /// 0x8DD061: `g_marksRingBefore`: each of the 0x100 indexes' one before it (`effects::make_profiles`).
    pub before: [PlainDword; 256],
    /// 0x8DD461: `g_marksRingAfter`: each index's one after it.
    pub after: [PlainDword; 256],
    /// 0x8DD861: `dword_8DD861`: 0 (`effects::make_profiles` writes it).
    pub zero: PlainDword,
}

impl ProfileRings {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        ProfileRings {
            before: [const { PlainDword::new(0x0) }; 256],
            after: [const { PlainDword::new(0x0) }; 256],
            zero: PlainDword::new(0x0),
        }
    }
    pub const START: usize = 0x8DD061;
    pub const LEN: usize = 0x804;
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

/// The profile tables (13 of 0x800 bytes, in this order of memory: ramp 0, curves 0 and 1, ramps 1 and 2, sines 0 .. 7) (IDA 0x8E506D .. 0x8EB86D).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ProfileTables {
    /// 0x8E506D: ramp 0, curve 0, curve 1, ramp 1, ramp 2, then sines 0 .. 7 (`RAMPS`, `CURVES`, `SINES`).
    pub tables: [ProfileTable; 13],
}

impl ProfileTables {
    /// The values the game starts with (the image's bytes).
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
    pub const START: usize = 0x8E506D;
    pub const LEN: usize = 0x6800;
}

#[cfg(test)]
mod typed_tests {
    use super::*;

    #[test]
    fn profile_rings_starts_as_the_games() {
        assert_eq!(size_of::<ProfileRings>(), ProfileRings::LEN);
        let image = crate::test_support::first_values::bytes(
            "effects.owner.EFFECTS.paths.profile_rings/2052",
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                (&raw const crate::effects::owner::effects().paths.profile_rings).cast::<u8>(),
                ProfileRings::LEN,
            )
        };
        assert!(
            mine == image,
            "PROFILE_RINGS starts differently from the image"
        );
    }

    /// The tables the image's pointer lists (0x9152F6) named, by address.
    #[test]
    fn table_numbers_name_the_tables_the_addresses_did() {
        let named = |ida: usize| (ida - ProfileTables::START) / 0x800;
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

    #[test]
    fn profile_tables_starts_as_the_games() {
        assert_eq!(size_of::<ProfileTables>(), ProfileTables::LEN);
        let key = "effects.owner.EFFECTS.paths.profile_tables/26624";
        let mine = unsafe {
            std::slice::from_raw_parts(
                (&raw const crate::effects::owner::effects().paths.profile_tables).cast::<u8>(),
                ProfileTables::LEN,
            )
        };
        assert!(
            crate::test_support::first_values::holds(key, mine),
            "PROFILE_TABLES starts differently from the image"
        );
    }
}
