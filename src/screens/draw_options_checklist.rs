//! `screens::draw_options_checklist` (0x47DEA2): a player's options page: its title (string
//! 0x190, a space and the player's number, in `byte_A8BC54`) at (0x190, 0x14), the page's
//! lines (strings 0x524 .. 0x531 at x 0x190 / 0x186, three fonts), the slider (`gfx::blit_bar`
//! at (0x128, 0x1E3), 0xD0 wide, `dword_75AC80` percent) with its knob and two labels, and a
//! tick or cross per option (`dword_759C76` .., -1 ends them; sprite 0x9E + (value ^ 1), its
//! palette the same value) at x 0x190 from y 0x82 every 0x1B (`dword_7559EC`, `hhmm` the
//! glyph, `g_markerX` 0x190).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x75927E). Compared in shadow mode (`dispatch_draw`).
//! Self-test: each player, options and slider values, nothing drawn (skipped without the
//! fonts).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_options_checklist",
    rva(0x47DEA2),
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

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

/// (id, x, y, font) of the page's lines.
/// The Rust version.
pub unsafe fn draw_options_checklist(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::settings_pages::options_checklist`.
    unsafe {
        let mut at = game_addr(rva(0x759C76)) as u32;
        let mut ticks = Vec::new();
        while (at as *const u32).read_unaligned() != u32::MAX {
            ticks.push((at as *const u32).read_unaligned());
            at += 4;
        }
        rc2k_core::screens::settings_pages::options_checklist(
            &mut crate::screens::game_screen::SettingsScreen::new(canvas),
            k(0x75598C),
            k(0x75AC80) as i32,
            &ticks,
        )
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_options_checklist(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    capture(&[
        g(0xA8BC54, 0x100),
        g(0x7559AC, 0x48),
        g(0x755A18, 4),
        (crate::strings::text_from_ids::text() as u32, 0x100),
        g(0x4F5040, 4),
    ])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let regions: Vec<Region> = vec![
        at(rva(0x75598C)),
        at(rva(0x75AC80)),
        g(0x759C76, 0x40),
        g(0x755A00, 0x30),
    ];
    let saved = capture(&regions);
    let bar = crate::gfx::blit_bar::Bar::read();
    for (player, percent, options) in [(0u32, 50u32, 3usize), (3, 100, 0), (1, 1, 14)] {
        check_draw(
            &mut t,
            &format!("player {player} percent {percent} options {options}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved);
                game_write(rva(0x75598C), player);
                game_write(rva(0x75AC80), percent);
                for n in 0..0xF {
                    let v = if n < options {
                        (n as u32) & 1
                    } else {
                        u32::MAX
                    };
                    game_write(rva(0x759C76) + 4 * n, v);
                }
            },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    bar.write();
    t.finish()
}
