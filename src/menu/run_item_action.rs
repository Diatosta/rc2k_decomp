//! `menu::run_item_action` (0x474683): calls the action of the highlighted item of a menu.
//!
//! In:  edi = the menu
//! Out: every register preserved
//! It goes through the menu's items (0x5C bytes each after a 4-byte head, ended by -1) until
//! one's type (+0x10) is the highlighted item `dword_755525`. For that one it stores the
//! item's dword at +0x54 in `dword_755A38`, calls the item's action (+0x44) with that value
//! in eax and the item in ebp, and then sets `dword_755525` to the item's type again,
//! whatever the action did to it. No item is found: nothing happens.
//! The first item is looked at without being checked for the end mark.
//! The action is the game's and can run for as long as an event lasts, so the thunk is not
//! gated and shadow mode runs the original alone. 89 callers: every screen loop.
//! Refuses a menu without an end in its first 256 items.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::item_at::{ITEM_LEN, MENU_HEAD_LEN};
use crate::menu::run_list::{Call, in_game};
use crate::service::harness::{at, capture, restore};
use crate::service::state::MENU_ITEM_RVA;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::run_item_action", rva(0x474683), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

const ACTION_VALUE_RVA: usize = rva(0x755A38); // dword_755A38
const TYPE_OFFSET: u32 = 0x10;
const ACTION_OFFSET: u32 = 0x44;
const VALUE_OFFSET: u32 = 0x54;
const END: u32 = u32::MAX;
const ITEM_LIMIT: u32 = 256;

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

// ---- Rust version ----

/// `entry_regs` are the registers at entry: the action gets them, with eax and ebp set.
pub unsafe fn run_item_action(menu: u32, entry_regs: &Regs, call: Call) -> Result<(), BadInput> {
    unsafe {
        let highlighted: u32 = game_read(MENU_ITEM_RVA);
        let mut item = menu.wrapping_add(MENU_HEAD_LEN);
        for _ in 0..ITEM_LIMIT {
            if dword(item + TYPE_OFFSET) == highlighted {
                let value = dword(item + VALUE_OFFSET);
                game_write(ACTION_VALUE_RVA, value);
                let mut g = *entry_regs;
                (g.eax, g.ebp) = (value, item);
                call(dword(item + ACTION_OFFSET), &mut g);
                game_write(MENU_ITEM_RVA, dword(item + TYPE_OFFSET));
                return Ok(());
            }
            item = item.wrapping_add(ITEM_LEN);
            if dword(item) == END {
                return Ok(());
            }
        }
    }
    Err(BadInput("a menu without an end"))
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let entry_regs = *r;
    if shadow() || unsafe { run_item_action(r.edi, &entry_regs, &mut in_game) }.is_err() {
        unsafe { call_original(r) };
    }
}

// ---- self-test: a menu of its own whose actions are a stand-in ----

static ACTIONS: AtomicU32 = AtomicU32::new(0);
static ACTION_EAX: AtomicU32 = AtomicU32::new(0);
static ACTION_EBP: AtomicU32 = AtomicU32::new(0);
static HIGHLIGHT_ADDR: AtomicU32 = AtomicU32::new(0);

/// Stands in for an item's action: notes eax and ebp, and moves the highlight elsewhere, as
/// an action may.
#[unsafe(naked)]
unsafe extern "C" fn fake_action() {
    core::arch::naked_asm!(
        "mov [{eax}], eax",
        "mov [{ebp}], ebp",
        "inc dword ptr [{actions}]",
        "push ecx",
        "mov ecx, [{highlight}]",
        "mov dword ptr [ecx], 0x77",
        "pop ecx",
        "ret",
        eax = sym ACTION_EAX,
        ebp = sym ACTION_EBP,
        actions = sym ACTIONS,
        highlight = sym HIGHLIGHT_ADDR,
    )
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(MENU_ITEM_RVA), at(ACTION_VALUE_RVA)];
    let saved = capture(&regions);
    HIGHLIGHT_ADDR.store(regions[0].0, Ordering::SeqCst);

    // A head, three items of types 5, 9 and 2, and the end mark.
    let mut menu = vec![0xAAAA_AAAAu32; (4 + 3 * 0x5C + 4) / 4 + 1];
    for (n, (kind, value)) in [(5u32, 0x111u32), (9, 0x222), (2, 0x333)]
        .into_iter()
        .enumerate()
    {
        let item = 1 + n * 0x5C / 4;
        menu[item + 0x10 / 4] = kind;
        menu[item + 0x44 / 4] = fake_action as *const () as u32;
        menu[item + 0x54 / 4] = value;
    }
    menu[1 + 3 * 0x5C / 4] = END;
    let before = Regs {
        edi: menu.as_ptr() as u32,
        ..Regs::SENTINEL
    };
    let observed = || {
        (
            capture(&regions),
            [&ACTIONS, &ACTION_EAX, &ACTION_EBP].map(|v| v.load(Ordering::SeqCst)),
        )
    };

    for highlighted in [5u32, 9, 2, 7, 0] {
        let prepare = || {
            unsafe {
                game_write(MENU_ITEM_RVA, highlighted);
                game_write(ACTION_VALUE_RVA, 0x5EED_0001u32);
            }
            for v in [&ACTIONS, &ACTION_EAX, &ACTION_EBP] {
                v.store(0, Ordering::SeqCst);
            }
        };
        prepare();
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let theirs = observed();
        prepare();
        let outcome = unsafe { run_item_action(before.edi, &before, &mut in_game) };
        let mine = observed();
        let preserved = ro.preserved_except(&before, 0);
        t.check(outcome.is_ok() && theirs == mine && preserved, || {
            format!(
                "highlighted={highlighted}: original {theirs:x?} preserved={preserved}; rust {outcome:?} {mine:x?}"
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
