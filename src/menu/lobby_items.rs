//! The network game's three menu items (stubs in the menus' tables, 0x7491D4, 0x74911C,
//! 0x749178), each a call of the lobby and back:
//! - `menu::lobby_settings_item` (0x4AAADB): `net::settings_loop` (a 6-byte stub: `call;
//!   retn`);
//! - `menu::lobby_link_item_2` (0x4AAAE1): eax 2, `net::start_link`;
//! - `menu::lobby_link_item_1` (0x4AAAEC): eax 1, `net::start_link`.
//!
//! In:  nothing
//! Out: the registers as the lobby leaves them
//!
//! Item actions, run by `menu::run_item_action`; before 2026-10-19 not hooked, so the
//! standalone passed them by. The lobby through its hooks (`call_game`: their Rust
//! versions). Ungated thunks (the lobby runs its own screens); shadow mode runs the
//! original alone. The first has a manual patch with a private copy (the whole stub, its
//! call relocated); the others' first five bytes are the `mov`. Self-tests: copies whose
//! call goes to a stand-in, against the Rust versions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use std::sync::atomic::AtomicUsize;

const SETTINGS_LOOP: usize = 0x4168CF; // net::settings_loop
const START_LINK: usize = 0x416F50; // net::start_link

/// The Rust versions' one call (place 0).
fn game_part(_: u32, r: &mut Regs, callee: usize) -> Result<(), Refused> {
    unsafe { call_game(r, game_addr(rva(callee))) };
    Ok(())
}

/// A stub's Rust version: eax set (if `eax`), the call.
fn stub(entry: &Regs, eax: Option<u32>, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if let Some(v) = eax {
        r.eax = v;
    }
    part(0, &mut r)?;
    Ok(r)
}

fn test(
    t: &mut SelfTest,
    address: usize,
    len: usize,
    entry: &[u8],
    call: usize,
    callee: &'static [usize],
    eax: Option<u32>,
) {
    let copy = match copy(address, len, entry, &[call], callee, &[]) {
        Ok(c) => c,
        Err(why) => {
            t.check(false, || format!("no copy: {why}"));
            return;
        }
    };
    run_both(t, "once", &copy, &[], &|| {}, Regs::SENTINEL, false, &|r| {
        stub(r, eax, &mut test_part)
    });
}

pub mod settings {
    use super::*;

    static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

    pub static HOOK: RawHook =
        RawHook::new("menu::lobby_settings_item", rva(0x4AAADB), thunk, &ORIGINAL)
            .private_copy(6)
            .relocate_calls(&[0])
            .manual_patch()
            .self_test(self_test);

    hook_asm!(
        thunk,
        call_original,
        dispatch = dispatch,
        original = ORIGINAL,
        ungated
    );

    extern "C" fn dispatch(r: &mut Regs) {
        dispatch_parted(
            &HOOK,
            r,
            call_original,
            &|r, part| stub(r, None, part),
            |p, r| game_part(p, r, SETTINGS_LOOP),
        );
    }

    fn self_test() -> (usize, usize) {
        let Some(mut t) = SelfTest::begin(&HOOK) else {
            return (0, 0);
        };
        // call net::settings_loop
        let rel = (SETTINGS_LOOP as i64 - (0x4AAADB + 5)) as i32;
        let b = rel.to_le_bytes();
        test(
            &mut t,
            0x4AAADB,
            6,
            &[0xE8, b[0], b[1], b[2], b[3]],
            0,
            &[SETTINGS_LOOP],
            None,
        );
        t.finish()
    }
}

macro_rules! link_item {
    ($module:ident, $name:literal, $address:expr, $eax:expr) => {
        pub mod $module {
            use super::*;

            static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

            pub static HOOK: RawHook =
                RawHook::new($name, rva($address), thunk, &ORIGINAL).self_test(self_test);

            hook_asm!(
                thunk,
                call_original,
                dispatch = dispatch,
                original = ORIGINAL,
                ungated
            );

            extern "C" fn dispatch(r: &mut Regs) {
                dispatch_parted(
                    &HOOK,
                    r,
                    call_original,
                    &|r, part| stub(r, Some($eax), part),
                    |p, r| game_part(p, r, START_LINK),
                );
            }

            fn self_test() -> (usize, usize) {
                let Some(mut t) = SelfTest::begin(&HOOK) else {
                    return (0, 0);
                };
                // mov eax, n
                let n: u32 = $eax;
                let b = n.to_le_bytes();
                test(
                    &mut t,
                    $address,
                    11,
                    &[0xB8, b[0], b[1], b[2], b[3]],
                    5,
                    &[START_LINK],
                    Some($eax),
                );
                t.finish()
            }
        }
    };
}

link_item!(link_2, "menu::lobby_link_item_2", 0x4AAAE1, 2);
link_item!(link_1, "menu::lobby_link_item_1", 0x4AAAEC, 1);
