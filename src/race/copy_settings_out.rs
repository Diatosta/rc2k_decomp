//! `race::copy_settings_out` (0x4486FB): "COPY SHAUN TO GRAHAM": after a race (not a
//! replay's, `dword_5BEC94`) the race's variables and the two players' blocks back into
//! the front end's (copies, floats to whole numbers), the results by mode
//! (`race::store_results_by_mode`), each logged.
//!
//! In:  nothing
//! Out: eax as the steps leave it, the rest as `race::store_results_by_mode`
//!
//! See `race/settings_copy.rs` (the steps a table made from the disassembly by script).
//! One caller, `sub_443F0B` (its last step; no flags read). Shadow mode runs the original
//! alone (it logs). Self-test: a copy whose 228 calls go to stand-ins, against the Rust
//! version on the same: every dword it reads set eight ways, one and two players, a
//! replay's and not.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::settings_copy::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::copy_settings_out", rva(0x4486FB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { copy_settings_out(r, &mut game_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { copy_settings_out(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

const CALL_OFFSETS: &[usize] = &[
    0x7, 0x14, 0x507, 0x557, 0x563, 0x570, 0x57C, 0x589, 0x595, 0x5A2, 0x5B3, 0x5C0, 0x5D1, 0x5DE,
    0x5EF, 0x5FC, 0x60D, 0x61A, 0x62B, 0x638, 0x649, 0x656, 0x667, 0x674, 0x685, 0x692, 0x6A3,
    0x6B0, 0x6C1, 0x6CE, 0x6DF, 0x6EC, 0x6FD, 0x70A, 0x71B, 0x728, 0x739, 0x746, 0x757, 0x764,
    0x775, 0x782, 0x793, 0x7A0, 0x7B1, 0x7BE, 0x7CF, 0x7DC, 0x7ED, 0x7FA, 0x80B, 0x818, 0x829,
    0x836, 0x847, 0x854, 0x865, 0x872, 0x883, 0x890, 0x8A1, 0x8AE, 0x8BF, 0x8CC, 0x8DD, 0x8EA,
    0x8FB, 0x908, 0x919, 0x926, 0x937, 0x944, 0x955, 0x962, 0x973, 0x980, 0x991, 0x99E, 0x9AF,
    0x9BC, 0x9CD, 0x9DA, 0x9EB, 0x9F8, 0xA09, 0xA16, 0xA27, 0xA34, 0xA45, 0xA52, 0xA63, 0xA70,
    0xA81, 0xA8E, 0xA9F, 0xAAC, 0xABD, 0xACA, 0xADB, 0xAE8, 0xAF9, 0xB06, 0xB17, 0xB24, 0xB35,
    0xB42, 0xB53, 0xB60, 0xB71, 0xB7E, 0xB8F, 0xBA9, 0xBB5, 0xBC2, 0xBCE, 0xBDB, 0xBE7, 0xBF4,
    0xC05, 0xC12, 0xC23, 0xC30, 0xC41, 0xC4E, 0xC5F, 0xC6C, 0xC7D, 0xC8A, 0xC9B, 0xCA8, 0xCB9,
    0xCC6, 0xCD7, 0xCE4, 0xCF5, 0xD02, 0xD13, 0xD20, 0xD31, 0xD3E, 0xD4F, 0xD5C, 0xD6D, 0xD7A,
    0xD8B, 0xD98, 0xDA9, 0xDB6, 0xDC7, 0xDD4, 0xDE5, 0xDF2, 0xE03, 0xE10, 0xE21, 0xE2E, 0xE3F,
    0xE4C, 0xE5D, 0xE6A, 0xE7B, 0xE88, 0xE99, 0xEA6, 0xEB7, 0xEC4, 0xED5, 0xEE2, 0xEF3, 0xF00,
    0xF11, 0xF1E, 0xF2F, 0xF3C, 0xF4D, 0xF5A, 0xF6B, 0xF78, 0xF89, 0xF96, 0xFA7, 0xFB4, 0xFC5,
    0xFD2, 0xFE3, 0xFF0, 0x1001, 0x100E, 0x101F, 0x102C, 0x103D, 0x104A, 0x105B, 0x1068, 0x1079,
    0x1086, 0x1097, 0x10A4, 0x10B5, 0x10C2, 0x10D3, 0x10E0, 0x10F1, 0x10FE, 0x110F, 0x111C, 0x112D,
    0x113A, 0x114B, 0x1158, 0x1169, 0x1176, 0x1187, 0x1194, 0x11A5, 0x11B2, 0x11C3, 0x11D0, 0x11E1,
    0x11EE, 0x11FA, 0x1207, 0x1213, 0x1220, 0x122C, 0x123B, 0x1248, 0x1255,
];
const PLACES: &[(usize, u32)] = &[
    (0x401427, WRITE),
    (0x40142B, WRITE_NL),
    (0x401476, INT_NL),
    (0x401626, HEX8_NL),
    (0x4015D6, FLOAT_NL),
    (0x449D44, 6),
    (0x4650B7, 7),
    (0x449957, 8),
];

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4486FB,
        0x125C,
        &[0x60, 0x8D, 0x05, 0x20, 0xD0, 0x4D, 0x00],
        CALL_OFFSETS,
        PLACES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut dwords = written(&SHAUN_TO_GRAHAM);
    dwords.extend(read(&SHAUN_TO_GRAHAM));
    dwords.sort();
    dwords.dedup();
    let regions = regions_of(&dwords);
    let saved = capture(&regions);
    // Values round the steps' tests and the floats' (0, 50, 99, 100, -5, 1.5, -1).
    let values = [0u32, 50, 99, 100, 0xFFFF_FFFB, 0x3FC0_0000, 0xFFFF_FFFF, 7];
    for (k, players) in [1u32, 2, 1, 2, 2, 1, 2, 1].into_iter().enumerate() {
        run_both(
            &mut t,
            &format!("values from {k}, {players} players"),
            &copy,
            &regions,
            &|| unsafe {
                for (n, &(a, _)) in regions.iter().enumerate() {
                    (a as *mut u32).write_unaligned(values[(n + k) % values.len()]);
                }
                crate::framework::game_write(rva(0x63DB94), players);
                crate::framework::game_write(rva(0x5BEC94), (k == 3) as u32);
            },
            &|r| unsafe { copy_settings_out(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
