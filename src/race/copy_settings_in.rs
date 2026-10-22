//! `race::copy_settings_in` (0x446EBE): "COPY GRAHAM TO SHAUN": the front end's settings
//! (0x75ACxx ..) into the race's variables and the two players' blocks (copies, whole
//! numbers to floats, the brakes and others held to 0 .. 0x100, the damage bits), each
//! logged; the two players' absences cleared; the league table copied.
//!
//! In:  nothing
//! Out: eax and ebx as the steps leave them, the rest as `race::copy_league_table`
//!
//! See `race/settings_copy.rs` (the steps a table made from the disassembly by script).
//! One caller, `sub_443F0B` (`race::set_up_arcade_cars` follows; no flags read). Shadow
//! mode runs the original alone (it logs). Self-test: a copy whose 260 calls go to
//! stand-ins, against the Rust version on the same: every dword it reads set to values
//! round its tests eight ways, one and two players, the damage bits given and not.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::settings_copy::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::copy_settings_in", rva(0x446EBE), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { copy_settings_in(r, &mut game_part) } {
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
        let g = unsafe { copy_settings_in(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

const CALL_OFFSETS: &[usize] = &[
    0x7, 0x14, 0x117, 0x123, 0x130, 0x13C, 0x149, 0x155, 0x162, 0x16E, 0x17B, 0x187, 0x194, 0x1A0,
    0x1AD, 0x1B9, 0x1C6, 0x1D2, 0x1DF, 0x1EB, 0x1F8, 0x204, 0x211, 0x21D, 0x22A, 0x236, 0x243,
    0x24F, 0x25C, 0x268, 0x4C7, 0x4D3, 0x4E0, 0x4EC, 0xB29, 0xB35, 0xB42, 0xB4E, 0xB5B, 0xB67,
    0xB74, 0xB85, 0xB92, 0xBA3, 0xBB0, 0xBC1, 0xBCE, 0xBDF, 0xBEC, 0xBFD, 0xC0A, 0xC1B, 0xC28,
    0xC39, 0xC46, 0xC57, 0xC64, 0xC75, 0xC82, 0xC93, 0xCA0, 0xCB1, 0xCBE, 0xCCF, 0xCDC, 0xCED,
    0xCFA, 0xD0B, 0xD18, 0xD29, 0xD36, 0xD47, 0xD54, 0xD65, 0xD72, 0xD83, 0xD90, 0xDA1, 0xDAE,
    0xDBF, 0xDCC, 0xDDD, 0xDEA, 0xDFB, 0xE08, 0xE19, 0xE26, 0xE37, 0xE44, 0xE55, 0xE62, 0xE73,
    0xE80, 0xE91, 0xE9E, 0xEAF, 0xEBC, 0xECD, 0xEDA, 0xEEB, 0xEF8, 0xF09, 0xF16, 0xF27, 0xF34,
    0xF45, 0xF52, 0xF63, 0xF70, 0xF81, 0xF8E, 0xF9F, 0xFAC, 0xFBD, 0xFCA, 0xFDB, 0xFE8, 0xFF9,
    0x1006, 0x1017, 0x1024, 0x1035, 0x1042, 0x1053, 0x1060, 0x1071, 0x107E, 0x108F, 0x109C, 0x10AD,
    0x10BA, 0x10CB, 0x10D8, 0x10E9, 0x10F6, 0x1107, 0x1114, 0x1125, 0x1132, 0x1143, 0x1150, 0x1161,
    0x117B, 0x1187, 0x1194, 0x11A0, 0x11AD, 0x11B9, 0x11C6, 0x11D7, 0x11E4, 0x11F5, 0x1202, 0x1213,
    0x1220, 0x1231, 0x123E, 0x124F, 0x125C, 0x126D, 0x127A, 0x128B, 0x1298, 0x12A9, 0x12B6, 0x12C7,
    0x12D4, 0x12E5, 0x12F2, 0x1303, 0x1310, 0x1321, 0x132E, 0x133F, 0x134C, 0x135D, 0x136A, 0x137B,
    0x1388, 0x1399, 0x13A6, 0x13B7, 0x13C4, 0x13D5, 0x13E2, 0x13F3, 0x1400, 0x1411, 0x141E, 0x142F,
    0x143C, 0x144D, 0x145A, 0x146B, 0x1478, 0x1489, 0x1496, 0x14A7, 0x14B4, 0x14C5, 0x14D2, 0x14E3,
    0x14F0, 0x1501, 0x150E, 0x151F, 0x152C, 0x153D, 0x154A, 0x155B, 0x1568, 0x1579, 0x1586, 0x1597,
    0x15A4, 0x15B5, 0x15C2, 0x15D3, 0x15E0, 0x15F1, 0x15FE, 0x160F, 0x161C, 0x162D, 0x163A, 0x164B,
    0x1658, 0x1669, 0x1676, 0x1687, 0x1694, 0x16A5, 0x16B2, 0x16C3, 0x16D0, 0x16E1, 0x16EE, 0x16FF,
    0x170C, 0x171D, 0x172A, 0x173B, 0x1748, 0x1759, 0x1766, 0x1777, 0x1784, 0x1795, 0x17A2, 0x17B3,
    0x17B9, 0x17D9, 0x17E5, 0x17F2, 0x17FE, 0x180B, 0x1817, 0x181D, 0x1829, 0x1836,
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
        0x446EBE,
        0x183D,
        &[0x60, 0x8D, 0x05, 0x9A, 0xC6, 0x4D, 0x00],
        CALL_OFFSETS,
        PLACES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut dwords = written(&GRAHAM_TO_SHAUN);
    dwords.extend(read(&GRAHAM_TO_SHAUN));
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
                if k % 2 == 0 {
                    crate::framework::game_write(rva(0x75B00C), u32::MAX);
                    crate::framework::game_write(rva(0x75B010), u32::MAX);
                }
            },
            &|r| unsafe { copy_settings_in(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
