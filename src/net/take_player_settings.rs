//! `net::take_player_settings` (0x48755D): player 1's record (`g_players[0]`) copied into
//! the network game's settings (`dword_75ACE8` .. `byte_75AE87`): 91 dwords (the car,
//! the driving aids and the HUD's switches, the controls, the co-driver, the view; the
//! dword at `dword_75AD8C` twice, the second one kept), 12 bytes from +0x1C, 0x20 bytes from
//! +0x24C to `dword_75B06C`; and the record's +0xD0 made 1.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! A null `g_players[0]` (the original reads through it) is refused. Caller:
//! `net::start_link`. Shadow mode: both run, the settings and the record compared.
//! Self-test: a copy against the Rust version, on a test record put in `g_players[0]`.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{Part, Saved, a, dword, fill, g, movsb, p, pb, region, run_both};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::take_player_settings", rva(0x48755D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PLAYERS: usize = 0x75647C; // g_players
const BLOCK_TO: usize = 0x75B06C; // dword_75B06C: 0x20 bytes
/// The block's place among the dwords: after the eighth.
const BLOCK_AFTER: usize = 8;
const SETTINGS: usize = 0x75ACE8;
const SETTINGS_LEN: usize = 0x3A8;

/// The settings: (to, the record's offset), in the original's order.
const DWORDS: [(usize, u32); 91] = [
    (0x75ACE8, 0x8),
    (0x75AD1C, 0x10C),
    (0x75AD20, 0x110),
    (0x75AD24, 0x114),
    (0x75AD28, 0x118),
    (0x75AD2C, 0x11C),
    (0x75AD30, 0x120),
    (0x75B004, 0x288),
    (0x75B014, 0x278),
    (0x75AD34, 0xD4),
    (0x75AD38, 0xD8),
    (0x75AD3C, 0xDC),
    (0x75AD40, 0xE0),
    (0x75AD44, 0xE4),
    (0x75AD48, 0xE8),
    (0x75AD4C, 0xEC),
    (0x75AD50, 0xF0),
    (0x75AD54, 0xF4),
    (0x75AD58, 0xF8),
    (0x75AD5C, 0xFC),
    (0x75AD60, 0x100),
    (0x75AD8C, 0x104),
    (0x75AD64, 0x108),
    (0x75ADF8, 0x158),
    (0x75ADFC, 0x15C),
    (0x75AE00, 0x160),
    (0x75AE08, 0x168),
    (0x75AE0C, 0x164),
    (0x75AE14, 0x170),
    (0x75AE10, 0x16C),
    (0x75AE18, 0x174),
    (0x75AE04, 0x178),
    (0x75B00C, 0x284),
    (0x75AD68, 0x17C),
    (0x75AD6C, 0x180),
    (0x75AD70, 0x184),
    (0x75AD74, 0x188),
    (0x75AD78, 0x18C),
    (0x75AD7C, 0x190),
    (0x75AD80, 0x194),
    (0x75AD84, 0x198),
    (0x75AD88, 0x19C),
    (0x75AD8C, 0x1A0),
    (0x75AD90, 0x1A4),
    (0x75AD94, 0x1A8),
    (0x75AD98, 0x1AC),
    (0x75AD9C, 0x1B0),
    (0x75ADA0, 0x1B4),
    (0x75ADA4, 0x1B8),
    (0x75ADA8, 0x1BC),
    (0x75ADAC, 0x1C0),
    (0x75ADB0, 0x1C4),
    (0x75ADB4, 0x1C8),
    (0x75ADB8, 0x1CC),
    (0x75ADBC, 0x1D0),
    (0x75ADC0, 0x1D4),
    (0x75ADC4, 0x1D8),
    (0x75ADC8, 0x1DC),
    (0x75ADCC, 0x1E0),
    (0x75ADD0, 0x1E4),
    (0x75ADD4, 0x1E8),
    (0x75ADD8, 0x1EC),
    (0x75ADDC, 0x1F0),
    (0x75ADE0, 0x1F4),
    (0x75ADE4, 0x1F8),
    (0x75ADE8, 0x1FC),
    (0x75ADEC, 0x200),
    (0x75ADF0, 0x204),
    (0x75ADF4, 0x208),
    (0x75AE1C, 0x28),
    (0x75AE20, 0x154),
    (0x75AE24, 0xC),
    (0x75AE28, 0x2C),
    (0x75AE2C, 0x30),
    (0x75AE30, 0x34),
    (0x75AE34, 0x38),
    (0x75AE38, 0x3C),
    (0x75AE3C, 0x40),
    (0x75AE40, 0x44),
    (0x75AE44, 0x48),
    (0x75AE4C, 0x50),
    (0x75AE50, 0x54),
    (0x75AE54, 0x58),
    (0x75AE58, 0x5C),
    (0x75AE5C, 0x60),
    (0x75AE60, 0x64),
    (0x75AE64, 0x68),
    (0x75AE68, 0x6C),
    (0x75AE6C, 0x70),
    (0x75AE74, 0x74),
    (0x75AE78, 0x78),
];
/// The bytes: (to, the record's offset).
const BYTES: [(usize, u32); 12] = [
    (0x75AE7C, 0x1C),
    (0x75AE7D, 0x1D),
    (0x75AE7E, 0x1E),
    (0x75AE7F, 0x1F),
    (0x75AE80, 0x20),
    (0x75AE81, 0x21),
    (0x75AE82, 0x22),
    (0x75AE83, 0x23),
    (0x75AE84, 0x24),
    (0x75AE85, 0x25),
    (0x75AE86, 0x26),
    (0x75AE87, 0x27),
];

/// The Rust version: the registers it leaves (the entry's).
pub fn take_player_settings(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let record = g(PLAYERS);
    if record == 0 {
        return Err(BadInput("no record for player 1").into());
    }
    let get = |offset: u32| unsafe { ((record + offset) as *const u32).read_unaligned() };
    unsafe { ((record + 0xD0) as *mut u32).write_unaligned(1) };
    for (n, &(to, from)) in DWORDS.iter().enumerate() {
        if n == BLOCK_AFTER {
            movsb(a(BLOCK_TO), record + 0x24C, 0x20);
        }
        p(to, get(from));
    }
    for (to, from) in BYTES {
        pb(to, get(from) as u8);
    }
    Ok(*entry)
}

/// What it writes: the settings and the record's +0xD0.
fn regions() -> Vec<crate::service::harness::Region> {
    let mut all = vec![region(SETTINGS, SETTINGS_LEN)];
    if g(PLAYERS) != 0 {
        all.push((g(PLAYERS) + 0xD0, 4));
    }
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::race::viewer::dispatch_regs(&HOOK, r, call_original, &regions(), 0, &|r| {
        take_player_settings(r, &mut crate::net::lobby::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; push eax; mov eax, 0
    let copy = match crate::net::lobby::copy(
        0x48755D,
        0x43C,
        &[0x60, 0x50, 0xB8, 0x00, 0x00, 0x00, 0x00],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut record: Vec<u32> = (0..0xC0u32)
        .map(|n| n.wrapping_mul(0x0101_0101) ^ 0x5A5A_0000)
        .collect();
    let at = record.as_mut_ptr() as u32;
    let all = [region(SETTINGS, SETTINGS_LEN), dword(PLAYERS), (at, 0x300)];
    let saved = Saved::new(&all);
    run_both(
        &mut t,
        "a test record",
        copy.addr(),
        Regs::SENTINEL,
        &all,
        0,
        &[],
        &|| {
            fill(SETTINGS, SETTINGS_LEN, 41);
            p(PLAYERS, at);
            unsafe { ((at + 0xD0) as *mut u32).write_unaligned(7) };
        },
        &take_player_settings,
    );
    drop(saved);
    let _ = &mut record;
    t.finish()
}
