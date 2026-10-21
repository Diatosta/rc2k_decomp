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
use crate::net::lobby::{Part, Saved, a, dword, fill, g, movsb, p, region, run_both};
use rc2k_core::net::settings::{self, Copy};
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
const SETTINGS: usize = 0x75ACE8; // dword_75ACE8
const SETTINGS_LEN: usize = settings::LEN;

/// The Rust version: the registers it leaves (the entry's).
pub fn take_player_settings(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let record = g(PLAYERS);
    if record == 0 {
        return Err(BadInput("no record for player 1").into());
    }
    // Converted: `rc2k_core::net::settings` (`copies`, `TAKEN_MARK`).
    unsafe { ((record + settings::TAKEN_MARK) as *mut u32).write_unaligned(1) };
    let to = a(SETTINGS);
    for copy in settings::copies() {
        match copy {
            Copy::Dword { to: at, from } => {
                let v = unsafe { ((record + from) as *const u32).read_unaligned() };
                unsafe { ((to + at) as *mut u32).write_unaligned(v) };
            }
            Copy::Bytes { to: at, from, len } => movsb(to + at, record + from, len),
        }
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
