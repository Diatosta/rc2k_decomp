//! `effects::make_profiles` (0x4C3B01), the body of `init_marks`' step 3 (`sub_4C4E1C` is
//! only its call; IDA, 2026-10-16): two rings of 0x100 indexes, once the 13 profile
//! tables (0x100 whole numbers and their 0x100 differences each), then the corners from
//! the course and `allocate_links`, `reset_settings_blocks`.

use crate::effects::blocks::MARK_CORNERS;
use crate::effects::frame_state::{DRAW_FRAME, DRAW_WORK};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::session::Dword;

pub const RESUME: u32 = 0x4C3EA6; // loc_4C3EA6
pub const COURSE_CORNERS: u32 = 0x1C_2000;
pub const CORNERS_LEN: usize = 0x8000;

/// A table the image holds by IDA's address (the pointer lists at 0x9152F6): the table in
/// `PROFILE_TABLES` it names. Stops the program for an address that is not a table's start.
fn ida_table(ida: u32) -> &'static ProfileTable {
    let (start, len) = (ProfileTables::START, ProfileTables::LEN);
    assert!(
        (start..start + len).contains(&(ida as usize))
            && (ida as usize - start).is_multiple_of(0x800),
        "a pointer of the marks' data names {ida:#x}, outside the profile tables"
    );
    &PROFILE_TABLES.tables[(ida as usize - start) / 0x800]
}

/// The profiles' work, `flt_8EB871` .. `flt_8EB899`.
fn work(n: usize, v: u32) {
    DRAW_WORK.profile_work[n].set_bits(v);
}

/// A table's second half: each the next less this (whole numbers, so the last reads the
/// second half's first, already a float), the first half made floats.
fn differences(table: &ProfileTable) {
    for n in 0..0x100 {
        table[0x100 + n].set(x87::int_less_int(
            table[n + 1].get() as i32,
            table[n].get() as i32,
        ));
        table[n].set(x87::from_int(table[n].get() as i32));
    }
}

/// The once-only tables.
fn make_tables() -> Result<(), Refused> {
    {
        for k in 0..3 {
            let t = ida_table(PROFILE_LISTS.ramps[k].get());
            let step = x87::int_less_int_over(
                t[1].get() as i32,
                t[0].get() as i32,
                DRAW_FRAME.profile_steps[0].bits(),
            );
            work(2, step);
            let mut v = x87::from_int(t[0].get() as i32);
            work(0, v);
            for cell in t.iter().take(0x100) {
                cell.set(x87::to_int(v) as u32);
                v = x87::add(v, step);
                work(0, v);
            }
            differences(t);
        }
        for k in 0..8 {
            let t = ida_table(PROFILE_LISTS.sines[k].get());
            let p: [u32; 4] = std::array::from_fn(|i| x87::loaded(t[i].get()));
            for (i, &v) in p.iter().enumerate() {
                work(3 + i, v);
            }
            let mut a = p[0];
            for cell in t.iter().take(0x100) {
                cell.set(x87::sine_times_plus_to_int(a, p[2], p[1]) as u32);
                a = x87::add(a, p[3]);
                work(3, a);
            }
            differences(t);
        }
        for k in 0..2 {
            let t = ida_table(PROFILE_LISTS.curves[k].get());
            let mut x = x87::loaded(t[0].get());
            work(7, x);
            let step = x87::sub_div(t[1].get(), t[0].get(), DRAW_FRAME.profile_steps[1].bits());
            work(8, step);
            let (scale, top) = (x87::loaded(t[2].get()), x87::loaded(t[3].get()));
            work(9, scale);
            work(10, top);
            for cell in t.iter().take(0x100) {
                cell.set(x87::negated_square_times_plus_to_int(x, scale, top) as u32);
                x = x87::add(x, step);
                work(7, x);
            }
            differences(t);
        }
    }
    Ok(())
}

/// `effects::make_profiles` (0x4C3B01): `dword_8DD861` 0, the rings' before and after (of
/// 0x100); unless done (`dword_91FF3E` = `resume`): the 3 ramps (start to end over 256
/// steps), 8 sines (base + amplitude sin, the angle stepped), 2 curves (top - x² scale),
/// each table's differences after it (a fourth list is of no tables), the mark `resume`;
/// then 0x8000 bytes of corners from `g_courseInfo` + 0x1C2000 (twice, as the original),
/// `allocate_links` and `reset_settings_blocks`. Refuses a mark that is neither 0 nor
/// `resume` (the original jumps to it). `resume` is `loc_4C3EA6`.
pub fn make_profiles(resume: u32) -> Result<(), Refused> {
    {
        let done: u32 = DRAW_FRAME.profiles_done.get();
        if done != 0 && done != resume {
            return Err(BadInput("the profiles' mark is neither 0 nor loc_4C3EA6").into());
        }
        PROFILE_RINGS.zero.set(0);
        for n in 0..0x100u32 {
            PROFILE_RINGS.before[n as usize].set(n.wrapping_sub(1) & 0xFF);
            PROFILE_RINGS.after[n as usize].set((n + 1) & 0xFF);
        }
        if done == 0 {
            make_tables()?;
            DRAW_FRAME.profiles_done.set(resume);
        }
        if COURSE_LISTS.course_info.bits() == 0 {
            return Err(BadInput("no course info (g_courseInfo 0)").into());
        }
        let corners = crate::race::course_places::info_bytes(
            COURSE_CORNERS,
            CORNERS_LEN,
            "the course corners",
        )?;
        GamePtr::to(&MARK_CORNERS.corners).write_bytes(&corners, "the course corners")?;
        crate::effects::blocks::allocate_links()?;
        crate::effects::blocks::reset_settings_blocks()?;
    }
    Ok(())
}

/// The profiles' rings of indexes (IDA 0x8DD061 .. 0x8DD865, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ProfileRings {
    /// 0x8DD061: `g_marksRingBefore`: each of the 0x100 indexes' one before it (`effects::make_profiles`).
    pub before: [Dword; 256],
    /// 0x8DD461: `g_marksRingAfter`: each index's one after it.
    pub after: [Dword; 256],
    /// 0x8DD861: `dword_8DD861`: 0 (`effects::make_profiles` writes it).
    pub zero: Dword,
}

pub static PROFILE_RINGS: ProfileRings = ProfileRings {
    before: [const { Dword::new(0x0) }; 256],
    after: [const { Dword::new(0x0) }; 256],
    zero: Dword::new(0x0),
};

impl ProfileRings {
    pub const START: usize = 0x8DD061;
    pub const LEN: usize = 0x804;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const PROFILE_RINGS) as usize
    }
}

/// The profile tables' address lists (IDA 0x9152F6 .. 0x91532A, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ProfileLists {
    /// 0x9152F6: `off_9152F6` (`g_marksRampTables`): the three ramp tables' addresses (IDA addresses in the image; `moved::RAW_KEPT` keeps them so).
    pub ramps: [Dword; 3],
    /// 0x915302: `off_915302` (`g_marksSineTables`): the eight sine tables' addresses.
    pub sines: [Dword; 8],
    /// 0x915322: `off_915322` (`g_marksCurveTables`): the two curve tables' addresses.
    pub curves: [Dword; 2],
}

pub static PROFILE_LISTS: ProfileLists = ProfileLists {
    ramps: [
        Dword::new(0x8E506D),
        Dword::new(0x8E686D),
        Dword::new(0x8E706D),
    ],
    sines: [
        Dword::new(0x8E786D),
        Dword::new(0x8E806D),
        Dword::new(0x8E886D),
        Dword::new(0x8E906D),
        Dword::new(0x8E986D),
        Dword::new(0x8EA06D),
        Dword::new(0x8EA86D),
        Dword::new(0x8EB06D),
    ],
    curves: [Dword::new(0x8E586D), Dword::new(0x8E606D)],
};

impl ProfileLists {
    pub const START: usize = 0x9152F6;
    pub const LEN: usize = 0x34;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const PROFILE_LISTS) as usize
    }
}

/// One profile table: 0x100 whole numbers and then their 0x100 differences (the first
/// dwords hold the table's inputs until `effects::make_profiles` makes it).
pub type ProfileTable = [Dword; 0x200];

/// A profile table with the given first values (index, value).
const fn table(first: &[(usize, u32)]) -> ProfileTable {
    let mut t = [const { Dword::new(0) }; 0x200];
    let mut k = 0;
    while k < first.len() {
        t[first[k].0] = Dword::new(first[k].1);
        k += 1;
    }
    t
}

/// The profile tables (13 of 0x800 bytes: three ramps, two curves, eight sines, in this order of memory) (IDA 0x8E506D .. 0x8EB86D, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ProfileTables {
    /// 0x8E506D: ramp 0, curve 0, curve 1, ramp 1, ramp 2, then sines 0 .. 7: the tables `ProfileLists` names.
    pub tables: [ProfileTable; 13],
}

pub static PROFILE_TABLES: ProfileTables = ProfileTables {
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
};

impl ProfileTables {
    pub const START: usize = 0x8E506D;
    pub const LEN: usize = 0x6800;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const PROFILE_TABLES) as usize
    }
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn profile_rings_starts_as_the_games() {
        assert_eq!(size_of::<ProfileRings>(), ProfileRings::LEN);
        let image = crate::moved::image_bytes(ProfileRings::START, ProfileRings::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(ProfileRings::home()).cast_const(),
                ProfileRings::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "PROFILE_RINGS starts differently from the image"
        );
        assert_eq!(
            offset_of!(ProfileRings, after),
            0x8DD461 - ProfileRings::START
        );
        assert_eq!(
            offset_of!(ProfileRings, zero),
            0x8DD861 - ProfileRings::START
        );
    }

    #[test]
    fn profile_lists_starts_as_the_games() {
        assert_eq!(size_of::<ProfileLists>(), ProfileLists::LEN);
        let image = crate::moved::image_bytes(ProfileLists::START, ProfileLists::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(ProfileLists::home()).cast_const(),
                ProfileLists::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "PROFILE_LISTS starts differently from the image"
        );
        assert_eq!(
            offset_of!(ProfileLists, sines),
            0x915302 - ProfileLists::START
        );
        assert_eq!(
            offset_of!(ProfileLists, curves),
            0x915322 - ProfileLists::START
        );
    }

    #[test]
    fn profile_tables_starts_as_the_games() {
        assert_eq!(size_of::<ProfileTables>(), ProfileTables::LEN);
        let image = crate::moved::image_bytes(ProfileTables::START, ProfileTables::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(ProfileTables::home()).cast_const(),
                ProfileTables::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "PROFILE_TABLES starts differently from the image"
        );
    }
}
