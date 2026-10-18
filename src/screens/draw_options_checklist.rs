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
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::font_at;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};
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
const LINES: [(u32, i32, i32, usize); 14] = [
    (0x524, 0x190, 0x64, 0x51E904),
    (0x525, 0x186, 0x82, 0x51E944),
    (0x526, 0x186, 0x9D, 0x51E944),
    (0x527, 0x186, 0xB8, 0x51E944),
    (0x528, 0x186, 0xD3, 0x51E8E4),
    (0x529, 0x186, 0xEE, 0x51E8E4),
    (0x52A, 0x186, 0x109, 0x51E8E4),
    (0x52B, 0x186, 0x124, 0x51E8E4),
    (0x52C, 0x186, 0x13F, 0x51E8E4),
    (0x52D, 0x186, 0x15A, 0x51E8E4),
    (0x52E, 0x186, 0x175, 0x51E8E4),
    (0x52F, 0x186, 0x190, 0x51E8E4),
    (0x530, 0x186, 0x1AB, 0x51E8E4),
    (0x531, 0x186, 0x1C6, 0x51E8E4),
];

/// The Rust version.
pub unsafe fn draw_options_checklist(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::gfx::blit_bar::{Bar, blit_bar};
    unsafe {
        let mut title = Vec::new();
        crate::strings::append_string_by_id::push_string(&mut title, 0x190);
        title.push(b' ');
        title.extend(crate::strings::append_u32::append_u32(
            k(0x75598C).wrapping_add(1),
        ));
        title.push(0);
        if title.len() > 0x100 {
            return Err(crate::errors::BadInput("a title past its buffer").into());
        }
        let buffer = game_addr(rva(0xA8BC54)) as *mut u8;
        std::ptr::copy_nonoverlapping(title.as_ptr(), buffer, title.len());
        draw_text_at(font_at(0x51E904), buffer, 0x190, 0x14, canvas)?;
        let mut palette = 0;
        for (id, x, y, font) in LINES {
            palette = draw_string_by_id_at(id, font_at(font), x, y, canvas)?.text as u32;
        }
        Bar {
            x: 0x128,
            y: 0x1E3,
            width: 0xD0,
            fill_glyph: 0x89,
            end_glyph: 0x87,
            percent: k(0x75AC80) as i32,
        }
        .write();
        blit_bar(&mut |call| canvas.blit_glyph(call))?;
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        canvas.blit_glyph(BlitCall::new(0x88, 0xEC, 0x1E2, palette, sprites))?;
        draw_string_by_id_at(0x4AE, font_at(0x51E3C4), 0x10A, 0x1E6, canvas)?;
        draw_string_by_id_at(0x4AF, font_at(0x51E3C4), 0x217, 0x1E6, canvas)?;
        game_write(rva(0x755A18), 0x190u32);
        let mut at = game_addr(rva(0x759C76)) as u32;
        let mut y = 0x82u32;
        while (at as *const u32).read_unaligned() != u32::MAX {
            game_write(rva(0x7559EC), y);
            let v = (at as *const u32).read_unaligned() ^ 1;
            let glyph = 0x9E_u32.wrapping_add(v);
            game_write(rva(0x7559F0), glyph);
            canvas.blit_glyph(BlitCall::new(
                glyph,
                k(0x755A18) as i32,
                y as i32,
                v,
                sprites,
            ))?;
            at += 4;
            y += 0x1B;
            game_write(rva(0x7559EC), y);
        }
        Ok(())
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
