//! `screens::draw_savegame_preview` (0x46D88F): draws the preview of the highlighted saved
//! game on the load-game screen, reading its file when the highlight has moved.
//!
//! In:  nothing
//! Out: every register preserved; carry as the panel drawn leaves it
//! `dword_755148` is the slot the buffer was last filled for and `dword_75514C` is 1 if there
//! is nothing to show. A highlighted item above 10 is no slot: the empty panel, and the
//! remembered slot becomes 999. Otherwise the slot is `dword_75523C * 10 + item - 1`; if it
//! is not the remembered one it is read with `read_savegame_slot`. Then
//! `draw_savegame_details` or `draw_empty_savegame`.
//! Its address is in the draw lists at 0x758BAE and 0x758BC6.
//!
//! Screen: the load-game screen ("CARREGAR CAMPEONATO"), the part under the ten slots, by the
//! owner's screenshot of 2026-10-10 and 1,455 shadow calls while it was open. Confirmed by
//! the owner with `blank on screens::draw_savegame_preview` the same day ("worked as
//! intended": the block under the slot list went away).
//! In shadow mode the two variables are put back after the Rust run, so that the original
//! reads the file as well.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::name_lists::SAVEGAME_PAGE_RVA;
use crate::saves::read_savegame_slot::read_savegame_slot;
use crate::saves::write_savegame_slot::{FILE_LEN, FILE_RVA};
use crate::screens::draw_empty_savegame::{draw_empty_savegame, preview_unavailable};
use crate::screens::draw_savegame_details::{details_unusable, draw_savegame_details};
use crate::screens::harness::{Left, check_draw, dispatch_draw_rewound, push_dwords};
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::{MENU_ITEM_RVA, menu_item};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_savegame_preview",
    rva(0x46D88F),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SHOWN_SLOT_RVA: usize = rva(0x755148); // dword_755148
const NOTHING_TO_SHOW_RVA: usize = rva(0x75514C); // dword_75514C

// ---- Rust version ----

/// Returns the carry flag of the panel drawn.
pub unsafe fn draw_savegame_preview(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::savegame_preview::preview` (the slot); the panels
    // are converted in their own hooks.
    use rc2k_core::screens::savegame_preview::{NO_SLOT, Preview, preview};
    unsafe {
        let page = game_read::<u32>(SAVEGAME_PAGE_RVA);
        match preview(menu_item(), page, game_read(SHOWN_SLOT_RVA)) {
            Preview::NoSlot => {
                game_write(NOTHING_TO_SHOW_RVA, 1u32);
                game_write(SHOWN_SLOT_RVA, NO_SLOT);
                return draw_empty_savegame(canvas);
            }
            Preview::Slot { slot, read: true } => {
                // Refused before anything is written: a slot past the list.
                let found = read_savegame_slot(slot)?;
                game_write(SHOWN_SLOT_RVA, slot);
                game_write(NOTHING_TO_SHOW_RVA, !found as u32);
            }
            Preview::Slot { .. } => {}
        }
        if game_read::<u32>(NOTHING_TO_SHOW_RVA) == 1 {
            draw_empty_savegame(canvas)
        } else {
            draw_savegame_details(canvas)
        }
    }
}

// ---- hook ----

fn decided_by() -> [Region; 2] {
    [at(SHOWN_SLOT_RVA), at(NOTHING_TO_SHOW_RVA)]
}

fn state() -> Vec<u8> {
    let mut s = crate::screens::draw_empty_savegame::state();
    push_dwords(&mut s, &[SHOWN_SLOT_RVA, NOTHING_TO_SHOW_RVA]);
    s
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_savegame_preview(canvas) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw_rewound(&HOOK, r, call_original, &rust, &state, &decided_by());
}

// ---- self-test: the owner's first saved game, an empty slot and "no slot" ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = preview_unavailable() {
        return t.skip(why);
    }
    let mut regions: Vec<Region> = vec![
        at(SHOWN_SLOT_RVA),
        at(NOTHING_TO_SHOW_RVA),
        at(SAVEGAME_PAGE_RVA),
        at(MENU_ITEM_RVA),
        (crate::framework::game_addr(FILE_RVA) as u32, FILE_LEN),
        (
            crate::strings::text_from_ids::text() as u32,
            crate::strings::text_from_ids::TEXT_LEN,
        ),
        (crate::screens::line_buffer() as u32, 0x80),
        (crate::files::numbered_path::buffer() as u32, 0x100),
        at(crate::files::read_file::LAST_FILE_PATH_RVA),
    ];
    // The names of the two slots the test reads, and the details panel's own variables.
    regions.push((crate::saves::write_savegame_slot::slot_name(0) as u32, 0x20));
    regions.push((
        crate::saves::write_savegame_slot::slot_name(99) as u32,
        0x20,
    ));
    regions.push((crate::framework::game_addr(rva(0x755150)) as u32, 0x20));
    let saved = capture(&regions);

    // (page, item, the slot remembered beforehand)
    for (what, page, item, shown) in [
        ("slot 0, to be read", 0u32, 1u32, 0x5EEDu32),
        ("slot 0, read already", 0, 1, 0),
        ("slot 99, to be read", 9, 10, 0x5EED),
        ("no slot", 0, 11, 0),
    ] {
        let prepare = || {
            restore(&regions, &saved);
            unsafe {
                game_write(SAVEGAME_PAGE_RVA, page);
                game_write(MENU_ITEM_RVA, item);
                game_write(SHOWN_SLOT_RVA, shown);
                game_write(NOTHING_TO_SHOW_RVA, 0u32);
            }
        };
        // "Read already" draws what the buffer holds: only if that can be drawn.
        prepare();
        if shown == 0 && item <= 10 && unsafe { details_unusable() }.is_some() {
            continue;
        }
        // For a slot that is read first, look at what the file holds.
        if shown == 0x5EED {
            let readable = unsafe { read_savegame_slot(page * 10 + item - 1) }.unwrap_or(false);
            if readable && unsafe { details_unusable() }.is_some() {
                continue;
            }
        }
        check_draw(
            &mut t,
            what,
            call_original,
            Regs::SENTINEL,
            &prepare,
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
