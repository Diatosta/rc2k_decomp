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
//! string table, and what its parts refuse.
//! Self-test: the four players, with and without a controller, typing and not, both
//! blinks, a second view of 5; a menu of the test's; nothing drawn.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::blit_marker_pair::sprite_table;
use crate::menu::disable_item::disable_item;
use crate::menu::enable_item::enable_item;
use crate::menu::item_at::{CURRENT_MENU_RVA, ITEM_LEN};
use crate::menu::player::{Player, player_at};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::draw_name_cursor::{BLINK_RVA, FONT, MEASURING_FONT, draw_name_cursor};
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::{LINE_BUFFER_RVA, font_at, line_buffer};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::format_u32::format_u32;
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};
use crate::text::measure_string_by_id::measure_string_by_id_at;
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
        if game_read::<u32>(CONTROLLERS_RVA) == 0 {
            disable_item(layout.controller_item)?;
        } else {
            enable_item(layout.controller_item)?;
        }
        let at = player_at(game_read(PLAYER_RVA));
        if at == 0 {
            return Err(BadInput("a player past the four").into());
        }
        let mut player = (at as *const Player).read_unaligned();
        if player.name == 0 {
            return Err(BadInput("a player without its name").into());
        }
        let font = font_at(FONT);
        let x = 0x1BB;
        let name = draw_text_at(font, player.name as *const u8, x, 0xD4, canvas)?;
        if game_read::<u32>(TYPING_RVA) != 0 {
            draw_name_cursor(at, name.x as u32, canvas)?;
        }
        let nationality = string(NATIONALITIES_RVA, player.nationality, NATIONALITIES)?;
        game_write(SECONDS_RVA, nationality);
        let drawn = draw_string_by_id_at(nationality, font, x, 0xEF, canvas)?;
        let measured =
            measure_string_by_id_at(nationality, font_at(MEASURING_FONT), drawn.x, canvas)?;
        canvas.blit_glyph(BlitCall::new(
            0x1C + player.nationality,
            measured.width.wrapping_add(0x1CA) as i32,
            0xF3,
            measured.lines,
            sprite_table(),
        ))?;
        let view = string(VIEWS_RVA, player.view, VIEWS)?;
        game_write(SECONDS_RVA, view);
        draw_string_by_id_at(view, font, x, 0x10A, canvas)?;
        // The lines below the views: one lower on a page with the second view.
        let mut y = 0x125;
        if layout.second_view {
            if player.second_view == 5 {
                player.second_view = 0;
                let mut written = (at as *const Player).read_unaligned();
                written.second_view = 0;
                (at as *mut Player).write_unaligned(written);
            }
            let second = string(SECOND_VIEWS_RVA, player.second_view, SECOND_VIEWS)?;
            game_write(SECONDS_RVA, second);
            draw_string_by_id_at(second, font, x, y, canvas)?;
            y += 0x1B;
        }
        let mut line = format_u32(player.race_number, 0);
        line.push(0);
        if line.len() > LINE_LEN {
            return Err(BadInput("a number longer than the line buffer").into());
        }
        std::ptr::copy_nonoverlapping(line.as_ptr(), line_buffer(), line.len());
        draw_text_at(font, line_buffer(), x, y, canvas)?;
        let codriver_sex = player.codriver_sex.wrapping_add(0x18A);
        game_write(KEPT_RVA, codriver_sex);
        let last = draw_string_by_id_at(codriver_sex, font, x, y + 0x1B, canvas)?;
        // The blits' palette is ebx as the last text left it.
        let palette = last.text as u32;
        let bar = y + 0x36;
        canvas.blit_glyph(BlitCall::new(4, 0x1BA, bar, palette, sprite_table()))?;
        let marker = player
            .controller_sensitivity
            .wrapping_mul(2)
            .wrapping_add(0x1BA);
        game_write(KEPT_RVA, marker);
        canvas.blit_glyph(BlitCall::new(
            6,
            marker as i32,
            bar,
            palette,
            sprite_table(),
        ))?;
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
