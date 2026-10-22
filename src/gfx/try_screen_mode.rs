//! `gfx::try_screen_mode` (0x404AED): whether screen mode eax works: the mode's entry
//! looked up in the mode table (`g_screenModeTable`, `g_screenModeCount`, by its bits but
//! the third byte's low nibble's: `& 0xF0FFFF`, 0 failing), the mode set
//! (`gfx::set_screen_mode`), and with `g_screenModeFlags` bit 0x100000 a trial of the
//! textures (`gfx::allocate_textures` with eax 0x40, ebx 0x40, ecx 1, edx 2, ebp 0, then
//! `gfx::release_all_textures`). Meanwhile `dword_5ABA88` is 1 and the reports
//! (`dword_4F52AC`) off, both put back after.
//!
//! In:  eax = the mode
//! Out: carry = the mode does not work; else carry clear and the zero flag by the eax the
//!      last call left; every register kept (`pusha` .. `popa`)
//!
//! Callers: `gfx::choose_screen_modes` (its list of modes), `gfx::screen_mode_choices`,
//! `gfx::keep_race_screen_mode`, and two in the race's set-up (0x442D2C, 0x4432B6). Its
//! three calls through `part` (0 set the mode, 1 allocate, 2 release; in both programs
//! their hooks: the platform's in the standalone). Shadow mode runs the original alone
//! (it changes the screen). Self-test: a copy whose three calls go to stand-ins (each
//! failing or not), against the Rust version, on a mode table of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::viewer::{
    Part, Saved, answer, dispatch_parts, dword, get, put, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::try_screen_mode", rva(0x404AED), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data (IDA addresses) ----

pub const MODE_TABLE: usize = 0x4F4F3C; // g_screenModeTable: the modes, 8 bytes each, here
pub const MODE_COUNT: usize = 0x4F4F38; // g_screenModeCount
const MODE_FLAGS: usize = 0x4F4934; // g_screenModeFlags
const TRYING: usize = 0x5ABA88; // dword_5ABA88: 1 while a mode is tried
const REPORTS: usize = 0x4F52AC; // dword_4F52AC: the reports on

/// The calls by place, as IDA shows them.
const CALLEES: [usize; 3] = [
    0x404B63, // 0 gfx::set_screen_mode
    0x4051ED, // 1 gfx::allocate_textures
    0x4051D3, // 2 gfx::release_all_textures
];

// ---- Rust version ----

/// The Rust version: the registers and flags it leaves.
pub fn try_screen_mode(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let reports = get(REPORTS);
    put(REPORTS, 0);
    put(TRYING, 1);
    let mut r = *entry;
    // Ok(None): it works; Ok(Some(zero)): it does not, with the zero flag the failing step
    // left (`stc` keeps it).
    let failed = (|| -> Result<Option<bool>, Refused> {
        // Converted: `rc2k_core::gfx::screen_modes::look_up`.
        use rc2k_core::gfx::screen_modes::{Lookup, look_up};
        let count = get(MODE_COUNT);
        let table = game_addr(rva(MODE_TABLE)) as u32;
        let at = match look_up(
            unsafe { crate::system::find_entry::entries(table, count) },
            entry.eax,
        ) {
            Lookup::NoBits => return Ok(Some(true)),
            // `cmp ecx, 1` on a count of 0, else the `dec ecx` that reached 0.
            Lookup::Missing => return Ok(Some(count != 0)),
            Lookup::Found(n) => table.wrapping_add(8 * n as u32),
        };
        r.eax = entry.eax;
        r.ebx = at;
        r.edx = entry.eax;
        // The flags at the call: the search's match (`cmp edx, eax` equal, carry clear).
        r.set_carry(false);
        r.set_zero(true);
        part(0, &mut r)?;
        if r.carry() {
            return Ok(Some(r.eflags & crate::framework::ZERO_FLAG != 0));
        }
        if get(MODE_FLAGS) & 0x10_0000 != 0 {
            r.eax = 0x40;
            r.ebx = 0x40;
            r.ecx = 1;
            r.edx = 2;
            r.ebp = 0;
            // The flags at the call: `xor ebp, ebp`'s (zero, carry clear).
            r.set_carry(false);
            r.set_zero(true);
            part(1, &mut r)?;
            if r.carry() {
                return Ok(Some(r.eflags & crate::framework::ZERO_FLAG != 0));
            }
            part(2, &mut r)?;
        }
        Ok(None)
    })()?;
    put(TRYING, 0);
    put(REPORTS, reports);
    let mut out = *entry;
    out.set_carry(failed.is_some());
    out.set_zero(failed.unwrap_or(r.eax == 0));
    Ok(out)
}

/// In both programs: the three calls through their hooks.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::calls::call(rva(CALLEES[place as usize]), r);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        try_screen_mode(r, &mut game_part)
    });
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places: &'static [(usize, u32)] = &[(0x404B63, 0), (0x4051ED, 1), (0x4051D3, 2)];
    // pusha; push dword_4F52AC
    let copy = match crate::race::viewer::copy(
        0x404AED,
        0x76,
        &[0x60, 0xFF, 0x35, 0xAC, 0x52, 0x4F, 0x00],
        &[0x24, 0x2D, 0x53, 0x5A],
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [
        crate::race::viewer::region(MODE_TABLE, 0x20),
        dword(MODE_COUNT),
        dword(MODE_FLAGS),
        dword(TRYING),
        dword(REPORTS),
    ];
    let saved = Saved::new(&regions);
    // Four modes of the test's in the table's first places (put back after).
    let table: [u32; 8] = [0x32_1802, 0, 0x34_1880, 0, 0x31_1101, 0, 0x30_2203, 0];
    for mode in [0u32, 0x32_1802, 0x34_1880, 0x0F_0000, 0x75_1802, 0x36_4404] {
        for flags in [0u32, 0x10_0000] {
            for fails in 0..4u32 {
                run_both(
                    &mut t,
                    &format!("mode {mode:#x}, flags {flags:#x}, call {fails} fails"),
                    copy.addr(),
                    Regs {
                        eax: mode,
                        ..Regs::SENTINEL
                    },
                    &regions,
                    crate::framework::CARRY_FLAG | crate::framework::ZERO_FLAG,
                    &|| {
                        for (n, &v) in table.iter().enumerate() {
                            put(MODE_TABLE + 4 * n, v);
                        }
                        put(MODE_COUNT, 4);
                        put(MODE_FLAGS, flags);
                        put(TRYING, 0x77);
                        put(REPORTS, 3);
                        answer(Box::new(move |place, r| {
                            r.set_carry(place == fails);
                            r.eax = 0x100 * place;
                        }));
                    },
                    &|r| try_screen_mode(r, &mut test_part),
                );
            }
        }
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
