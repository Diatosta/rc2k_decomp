//! `screens::draw_player_details` (0x46F894): player 1's details page. The players'
//! pages share this file's code: `draw_player2_details` (0x4677B5) is the same code,
//! `draw_player34_details` (0x467975) the page without the second view (`Layout`).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! - The controller's item (9) of the current menu greyed out when there is no
//!   controller (`g_controllerCount`), else made choosable (`menu::disable_item`,
//!   `menu::enable_item`).
//! - The player of the player `dword_75598C` (`Player`): its name at (0x1BB, 0xD4) in
//!   the font at 0x51E8C4, and while a name is being typed (`dword_7556A8` not 0) the
//!   cursor after it (`screens::draw_name_cursor`).
//! - The nationality's string (`dword_758816[nationality]`, kept in `seconds`,
//!   0x7559AC) at (0x1BB, 0xEF); measured with the font at 0x51E824 from the x the text
//!   left, its flag (sprite 0x1C + nationality) at 0x1CA + the width, y 0xF3.
//! - The view `dword_7589BA[view]` at y 0x10A; the second view
//!   `dword_7589DA[second_view]` at 0x125, a second view of 5 made 0 first (only on the
//!   pages that have it; on the others the lines below are 0x1B higher).
//! - `race_number` as a number at 0x140; the co-driver's sex's string (0x18A + it, kept in
//!   `dword_7559EC`) at 0x15B.
//! - Sprite 4 at (0x1BA, 0x176) and sprite 6 at (0x1BA + 2 * `controller_sensitivity`,
//!   0x176), the
//!   blits' palette being what the last text left in ebx.
//!
//! In the draw list at 0x758CBA (the four players' details screens).
//! Refuses a player past the four, a player without its name, an index past its
//! string table, and what its parts refuse; since the conversion (2026-10-20) before the
//! controller's item is switched (the original switches it first).
//! Self-test: the four players, with and without a controller, typing and not, both
//! blinks, a second view of 5; a menu of the test's; nothing drawn.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::item_at::{CURRENT_MENU_RVA, ITEM_LEN};
use crate::menu::player::{Player, player_at};
use crate::screens::LINE_BUFFER_RVA;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::draw_name_cursor::{BLINK_RVA, MEASURING_FONT};
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::Canvas;
use std::cell::RefCell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_player_details",
    rva(0x46F894),
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

const CONTROLLERS_RVA: usize = rva(0x4F78E0); // g_controllerCount
const PLAYER_RVA: usize = rva(0x75598C); // dword_75598C
const TYPING_RVA: usize = rva(0x7556A8); // dword_7556A8
const SECONDS_RVA: usize = rva(0x7559AC); // seconds
const KEPT_RVA: usize = rva(0x7559EC); // dword_7559EC
const NATIONALITIES_RVA: usize = rva(0x758816); // dword_758816
const VIEWS_RVA: usize = rva(0x7589BA); // dword_7589BA
const SECOND_VIEWS_RVA: usize = rva(0x7589DA); // dword_7589DA
/// What differs between the players' pages: players 1 and 2 (`sub_46F894`,
/// `sub_4677B5`) have the controller's item at 9 and the second view's line; players 3
/// and 4 (`sub_467975`) have it at 8 and no second view, the lines below moving up one.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub controller_item: u32,
    pub second_view: bool,
}

pub const FULL: Layout = Layout {
    controller_item: 9,
    second_view: true,
};
pub const SHORT: Layout = Layout {
    controller_item: 8,
    second_view: false,
};
/// The tables' lengths (each ended by -1): 40 nationalities, seven views, six second
/// views.
const NATIONALITIES: u32 = 40;
const VIEWS: u32 = 7;
const SECOND_VIEWS: u32 = 6;
const LINE_LEN: usize = 0x100;

// ---- Rust version ----

fn string(table: usize, index: u32, count: u32) -> Result<u32, Refused> {
    if index >= count {
        return Err(BadInput("an index past its string table").into());
    }
    Ok(unsafe { game_read(table + index as usize * 4) })
}

pub unsafe fn draw_player_page(layout: Layout, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let at = player_at(game_read(PLAYER_RVA));
        if at == 0 {
            return Err(BadInput("a player past the four").into());
        }
        let player = (at as *const Player).read_unaligned();
        if player.name == 0 {
            return Err(BadInput("a player without its name").into());
        }
        // Converted: `rc2k_core::screens::player_details`; the strings looked up (and refused)
        // before anything is drawn, the second view of 5 made 0 then.
        let nationality_id = string(NATIONALITIES_RVA, player.nationality, NATIONALITIES)?;
        let view_id = string(VIEWS_RVA, player.view, VIEWS)?;
        let second_view_id = if layout.second_view {
            let view = if player.second_view == 5 {
                0
            } else {
                player.second_view
            };
            let id = string(SECOND_VIEWS_RVA, view, SECOND_VIEWS)?;
            if player.second_view == 5 {
                let mut written = (at as *const Player).read_unaligned();
                written.second_view = 0;
                (at as *mut Player).write_unaligned(written);
            }
            Some(id)
        } else {
            None
        };
        let details = rc2k_core::screens::player_details::Details {
            controller_item: layout.controller_item,
            controllers: game_read::<u32>(CONTROLLERS_RVA) != 0,
            name: player.name,
            typing: game_read::<u32>(TYPING_RVA) != 0,
            blink: game_read(crate::screens::draw_name_cursor::BLINK_RVA),
            nationality: player.nationality,
            nationality_id,
            view_id,
            second_view_id,
            race_number: player.race_number,
            codriver_sex: player.codriver_sex,
            sensitivity: player.controller_sensitivity,
        };
        rc2k_core::screens::player_details::player_details(
            &mut crate::screens::game_screen::DetailsScreen::new(canvas),
            &details,
        )?;
    }
    Ok(())
}

// ---- hook ----

fn left_behind(layout: Layout) -> Vec<Region> {
    let menu: u32 = unsafe { game_read(CURRENT_MENU_RVA) };
    let mut regions = vec![
        at(SECONDS_RVA),
        at(KEPT_RVA),
        (game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN),
        (text() as u32, TEXT_LEN),
    ];
    // The measuring font's leftovers are not compared: the shadow run's canvas does not
    // write font objects back (as for the other hooks that measure).
    if menu != 0 {
        regions.push((
            menu + 4 + (layout.controller_item - 1) * ITEM_LEN,
            ITEM_LEN as usize,
        ));
    }
    let index: u32 = unsafe { game_read(PLAYER_RVA) };
    let player = player_at(index);
    if player != 0 {
        regions.push((player, std::mem::size_of::<Player>()));
    }
    regions
}

/// A players' page hook's `dispatch`.
pub fn dispatch_page(
    hook: &RawHook,
    r: &mut Regs,
    call_original: unsafe extern "C" fn(*mut Regs),
    layout: Layout,
) {
    dispatch_draw(
        hook,
        r,
        call_original,
        &|_, canvas| {
            unsafe { draw_player_page(layout, canvas) }?;
            Ok(Left::nothing())
        },
        &|| capture(&left_behind(layout)),
    );
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_page(&HOOK, r, call_original, FULL);
}

// ---- self-test ----

/// A players' page hook's self-test: the four players under four settings each.
pub fn check_page(
    t: &mut SelfTest,
    call_original: unsafe extern "C" fn(*mut Regs),
    layout: Layout,
) {
    let inputs: [Region; 5] = [
        at(CONTROLLERS_RVA),
        at(PLAYER_RVA),
        at(TYPING_RVA),
        at(BLINK_RVA),
        at(CURRENT_MENU_RVA),
    ];
    let players: Vec<Region> = (0..4)
        .map(player_at)
        .filter(|&player| player != 0)
        .map(|player| (player, std::mem::size_of::<Player>()))
        .collect();
    let font = [(game_addr(rva(MEASURING_FONT)) as u32, 0x20)];
    let state = || capture(&left_behind(layout));
    let saved = (capture(&inputs), capture(&players), state(), capture(&font));
    let menu = RefCell::new(vec![0xAAu8; 4 + 12 * ITEM_LEN as usize]);
    let at_menu = menu.borrow_mut().as_mut_ptr() as u32;
    for index in 0..4u32 {
        let player = player_at(index);
        if player == 0 || unsafe { (player as *const Player).read_unaligned() }.name == 0 {
            continue;
        }
        let fresh = unsafe { (player as *const Player).read_unaligned() };
        for (controllers, typing, blink, second_five) in [
            (0u32, 0u32, 0u32, false),
            (2, 1, 1, false),
            (1, 1, 0, true),
            (0, 0, 1, true),
        ] {
            check_draw(
                t,
                &format!(
                    "player {index}, controllers {controllers}, typing {typing}, blink {blink}, second view 5 {second_five}"
                ),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    menu.borrow_mut().fill(0xAA);
                    game_write(CURRENT_MENU_RVA, at_menu);
                    game_write(CONTROLLERS_RVA, controllers);
                    game_write(PLAYER_RVA, index);
                    game_write(TYPING_RVA, typing);
                    game_write(BLINK_RVA, blink);
                    let mut fields = fresh;
                    if second_five {
                        fields.second_view = 5;
                    }
                    (player as *mut Player).write_unaligned(fields);
                },
                &|_, canvas| {
                    unsafe { draw_player_page(layout, canvas) }?;
                    Ok(Left::nothing())
                },
                &state,
            );
        }
    }
    restore(&inputs, &saved.0);
    restore(&players, &saved.1);
    restore(&left_behind(layout), &saved.2);
    restore(&font, &saved.3);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    check_page(&mut t, call_original, FULL);
    t.finish()
}
