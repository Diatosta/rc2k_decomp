//! `menu::list_step` (0x414A55): the highlight of a list menu (the replay viewer's,
//! `sub_4144CF`'s) moved by a key: 0x66 and 0x6B back and on, 0x68 and 0x69 the other way.
//! With a table of neighbours (ebx, 4 bytes an item: 0x66's, 0x6B's, 0x68's, 0x69's) the
//! item's entry, unless negative; without one (ebx 0) 0x66 and 0x6B go round the list
//! (`[edi+1]` items) and 0x68 and 0x69 do nothing. The item into `dword_51D407`.
//!
//! In:  al the key, ebx the table or 0, edi the list
//! Out: moved: ZF set, carry clear, ecx the item. Not moved: ZF clear; ecx kept for
//!      another key (carry: al below 0x69), else `dword_51D407` with cl the table's entry
//!      (carry clear), or for 0x68 / 0x69 without a table `dword_51D407` (carry set)
//!
//! Callers: `race::replay_viewer` (two), `sub_4144CF` (one), reading ZF. A loop target at
//! +4 (the patch would cut `jz`): private copy, manual patch. Shadow mode compares.
//! Self-test: the original against the Rust version, every key with and without a table,
//! items first, inside and last, table entries negative or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{CF, CHOICE, ZF, check_original, dispatch_regs, dword, get, put};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::list_step", rva(0x414A55), thunk, &ORIGINAL)
    .private_copy(0x74)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers and flags it leaves.
pub unsafe fn list_step(entry: &Regs) -> Regs {
    let mut r = *entry;
    let key = r.eax as u8;
    let table = r.ebx;
    let count = unsafe { ((r.edi + 1) as *const u8).read() };
    let entry_of = |ecx: u32, k: u32| unsafe {
        (table.wrapping_add(ecx.wrapping_mul(4)).wrapping_add(k) as *const u8).read()
    };
    let with_cl = |ecx: u32, cl: u8| ecx & !0xFF | cl as u32;
    let cl = match key {
        0x66 | 0x6B | 0x68 | 0x69 => {
            r.ecx = get(CHOICE);
            if (key == 0x68 || key == 0x69) && table < 1 {
                r.set_carry(true);
                r.set_zero(false);
                return r;
            }
            if table != 0 {
                let k = [0x66, 0x6B, 0x68, 0x69]
                    .iter()
                    .position(|&x| x == key)
                    .unwrap();
                let cl = entry_of(r.ecx, k as u32);
                if (cl as i8) < 0 {
                    r.ecx = with_cl(r.ecx, cl);
                    r.set_carry(false);
                    r.set_zero(false);
                    return r;
                }
                cl
            } else if key == 0x66 {
                let cl = (r.ecx as u8).wrapping_sub(1);
                if (cl as i8) < 0 {
                    count.wrapping_sub(1)
                } else {
                    cl
                }
            } else {
                let cl = (r.ecx as u8).wrapping_add(1);
                if cl < count { cl } else { 0 }
            }
        }
        _ => {
            r.set_carry(key < 0x69);
            r.set_zero(false);
            return r;
        }
    };
    r.ecx = with_cl(r.ecx, cl);
    put(CHOICE, r.ecx);
    r.set_carry(false);
    r.set_zero(true);
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(unsafe { list_step(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &[dword(CHOICE)], CF | ZF, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [dword(CHOICE)];
    let saved = crate::race::viewer::Saved::new(&regions);
    let mut list = [0xAAu8; 0x20];
    list[1] = 5;
    // Five items' neighbours, some negative.
    let table: [u8; 20] = [
        4, 1, 0xFF, 2, 0, 2, 3, 0xFE, 1, 3, 0x80, 4, 2, 4, 0, 1, 3, 0, 1, 0xFF,
    ];
    for key in [0x66u32, 0x6B, 0x68, 0x69, 0x10, 0x70, 0x6A] {
        for with_table in [false, true] {
            for choice in [0u32, 2, 4, 0x1_0003] {
                if with_table && choice > 4 {
                    continue; // the table has five items
                }
                let before = Regs {
                    eax: 0xAB00 | key,
                    ebx: if with_table { table.as_ptr() as u32 } else { 0 },
                    edi: list.as_mut_ptr() as u32,
                    ..Regs::SENTINEL
                };
                check_original(
                    &mut t,
                    &format!("key {key:#x}, table {with_table}, item {choice:#x}"),
                    call_original,
                    before,
                    &regions,
                    CF | ZF,
                    &|| put(CHOICE, choice),
                    &rust,
                );
            }
        }
    }
    drop(saved);
    t.finish()
}
