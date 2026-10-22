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
//! Caller: `physics::split_by_mode`.
//! Refuses a split or a split count past 0x40, a divisor of 0 or a quotient past a dword
//! (the original's `div` would fault); what was written is put back.
//! Self-test: `physics/splits.rs` (`test_splits`), mode 1.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::fill_rival_table::fill_rival_table_in;
use crate::physics::splits::{
    COUNT, FIGURE, SPLIT, TEXT, TIME, g, guarded, p, test_splits, written,
};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use crate::strings::append_string_by_id::append_string_by_id;
use rc2k_core::physics::splits::{RIVALS, gap, rank, scaled_time, split_scale};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::rank_split_time", rva(0x436C4F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A name into `text` (eight blanks first), its first eight bytes into `to`.
fn name(r: &mut Regs, id: u32, to: usize) {
    let text = crate::framework::game_addr(rva(TEXT)) as u32;
    p(TEXT, 0x2020_2020);
    p(TEXT + 4, 0x2020_2020);
    let mut cursor = text;
    r.eax = unsafe {
        append_string_by_id(id, &mut |s, n| {
            std::ptr::copy_nonoverlapping(s, cursor as *mut u8, n);
            cursor += n as u32;
        })
    };
    r.ebx = cursor;
    p(to, g(TEXT));
    p(to + 4, g(TEXT + 4));
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::splits::{split_scale, scaled_time, rank, gap}`.
pub fn rank_split_time_in(r: &mut Regs) -> Result<(), Refused> {
    let (split, count) = (g(SPLIT), g(COUNT));
    if split > 0x40 || count > 0x40 {
        return Err(BadInput("a split or a split count past 0x40").into());
    }
    fill_rival_table_in(r)?;
    for k in 0..5 {
        p(FIGURE + 4 * k, u32::MAX);
    }
    for at in [0x6D1838, 0x6D183C, 0x6D1844, 0x6D1848] {
        p(at, 0x2020_2020);
    }
    let base = g(0x601030 + 4 * split as usize);
    let d = g(0x60102C + 4 * count as usize);
    let (scale, rest) = split_scale(base, d)?;
    r.eax = scale;
    r.edx = rest;
    r.ebx = scale;
    for c in 0..RIVALS {
        r.ecx = c as u32;
        let (q, rest) = scaled_time(g(0x601ED0 + 4 * c), scale)?;
        r.eax = q;
        r.edx = rest;
        p(0x6021A0 + 4 * c, q);
    }
    r.ebp = 1000;
    // A loop whose work was cut out: only its registers are left.
    r.eax = 0x58 / 10;
    r.edx = 0x58 % 10;
    r.ebx = 10;
    let time = g(TIME);
    let (c, last) = rank(time, |c| g(0x6021A0 + 4 * c));
    r.eax = last;
    r.ecx = c;
    p(FIGURE, c);
    r.eax = time;
    p(0x602308, time);
    let c = c as usize;
    if c != 1 {
        // The one ahead: rival c - 2 (its scaled time, car and name).
        r.eax = gap(g(0x6021A0 + 4 * (c - 2)), time);
        p(FIGURE + 4, r.eax);
        r.eax = g(0x602038 + 4 * (c - 2));
        p(FIGURE + 0xC, r.eax);
        r.eax = g(0x601D68 + 4 * (c - 2));
        p(FIGURE + 0x14, r.eax);
        name(r, r.eax, 0x6D1844);
    }
    if c != RIVALS {
        // The one behind: rival c - 1.
        r.eax = gap(time, g(0x6021A0 + 4 * (c - 1)));
        p(FIGURE + 8, r.eax);
        r.eax = g(0x602038 + 4 * (c - 1));
        p(FIGURE + 0x10, r.eax);
        r.eax = g(0x601D68 + 4 * (c - 1));
        p(FIGURE + 0x18, r.eax);
        name(r, r.eax, 0x6D1838);
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn rank_split_time(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &rank_split_time_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { rank_split_time(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &written(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_splits(&mut t, call_original, &rust, &|c| c.mode == 1);
    t.finish()
}
