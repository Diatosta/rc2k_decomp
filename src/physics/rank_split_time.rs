//! `physics::rank_split_time` (0x436C4F, race mode 1): the car's place among the 90
//! rivals at this split. `physics::fill_rival_table`; the figures +0xA0C .. +0xA1C -1 and
//! the two name fields (`dword_6D1838` .., `dword_6D1844` ..) blanks; each rival's time
//! scaled to the split: `dword_601ED0[c]` x (`dword_601030[split]` x 1000 /
//! `dword_60102C[g_splitColourCount]`) / 1000 into `dword_6021A0[c]` (`mul` / `div`); the
//! place (+0xA0C) one past the first not below the car's time (90 at most);
//! `dword_602308` the time; but for first place the gap to the one ahead (x10) into
//! +0xA10, its car +0xA18 and name id +0xA20, the name (`strings::append_string_by_id`
//! into `text`) into `dword_6D1844` ..; but for last place the gap to the one behind into
//! +0xA14, its car +0xA1C, name id +0xA24 and name into `dword_6D1838` ...
//!
//! In:  nothing
//! Out: the registers as left (ebx the cursor after a name, ebp 1000); no flags read
//!
//! Caller: `physics::split_by_mode`. Refuses a split or a split count past 0x40, a divisor
//! of 0 or a quotient past a dword (the original's `div` would fault); what was written is
//! put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::physics::fill_rival_table::fill_rival_table;
use crate::physics::splits::{COUNT, FIGURE, SPLIT, TEXT, TIME, g, p};
use crate::strings::append_string_by_id::append_string_by_id;
use rc2k_core::physics::splits::{RIVALS, gap, rank, scaled_time, split_scale};

/// A name into `text` (eight blanks first), its first eight bytes into `to`.
fn name(id: u32, to: usize) {
    let text = crate::framework::game_addr(rva(TEXT)) as u32;
    p(TEXT, 0x2020_2020);
    p(TEXT + 4, 0x2020_2020);
    let mut cursor = text;
    unsafe {
        append_string_by_id(id, &mut |s, n| {
            std::ptr::copy_nonoverlapping(s, cursor as *mut u8, n);
            cursor += n as u32;
        })
    };
    p(to, g(TEXT));
    p(to + 4, g(TEXT + 4));
}

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::{split_scale, scaled_time, rank, gap}`.
pub fn rank_split_time() -> Result<(), Refused> {
    let (split, count) = (g(SPLIT), g(COUNT));
    if split > 0x40 || count > 0x40 {
        return Err(BadInput("a split or a split count past 0x40").into());
    }
    fill_rival_table()?;
    for k in 0..5 {
        p(FIGURE + 4 * k, u32::MAX);
    }
    for at in [0x6D1838, 0x6D183C, 0x6D1844, 0x6D1848] {
        p(at, 0x2020_2020);
    }
    let base = g(0x601030 + 4 * split as usize);
    let d = g(0x60102C + 4 * count as usize);
    let (scale, _) = split_scale(base, d)?;
    for c in 0..RIVALS {
        let (q, _) = scaled_time(g(0x601ED0 + 4 * c), scale)?;
        p(0x6021A0 + 4 * c, q);
    }
    // (A loop whose work was cut out: it left only registers, read by no one.)
    let time = g(TIME);
    let (c, _) = rank(time, |c| g(0x6021A0 + 4 * c));
    p(FIGURE, c);
    p(0x602308, time);
    let c = c as usize;
    if c != 1 {
        // The one ahead: rival c - 2 (its scaled time, car and name).
        p(FIGURE + 4, gap(g(0x6021A0 + 4 * (c - 2)), time));
        p(FIGURE + 0xC, g(0x602038 + 4 * (c - 2)));
        let id = g(0x601D68 + 4 * (c - 2));
        p(FIGURE + 0x14, id);
        name(id, 0x6D1844);
    }
    if c != RIVALS {
        // The one behind: rival c - 1.
        p(FIGURE + 8, gap(time, g(0x6021A0 + 4 * (c - 1))));
        p(FIGURE + 0x10, g(0x602038 + 4 * (c - 1)));
        let id = g(0x601D68 + 4 * (c - 1));
        p(FIGURE + 0x18, id);
        name(id, 0x6D1838);
    }
    Ok(())
}
