//! `screens::draw_section_best_times` (0x46762B; IDA had it as code outside any function,
//! made one 2026-10-19): ten rows of the section's best times (`off_742C98`[section], the
//! game's own tables of 0x9C-byte entries, six sections), from y 0xF0, 0x19 apart, row n
//! (1 .. 10, kept in `seconds`): a player's (+0x40 1) in 0x51E524 / 0x51E544, else 0x51E584 /
//! 0x51E5A4: the name (the text at +0x20) at 0xF0, the time (+0x18, `strings::append_time`,
//! drawn as text) at 0x1E0, n at 0xC8; then the nationality's sprite (0x1C + +8) at 0xD2,
//! the flag's (+0x10, 0x18 and 0x19 swapped, + 0x44) at 0x276, and again at 0x276 the glyph
//! that number gives once more (+0x44 on it, sic: `notes/bugs.md`), each with the palette
//! the last text left (ebx). Each drawer's y is edx as the text drawers leave it.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! In the best times screen's draw list (0x758E4E). Refuses (before anything) a section past
//! the six or with no table, and a line past the line buffer. Compared in shadow mode
//! (`dispatch_draw`). Self-test: each section, nothing drawn (skipped without the fonts).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::{LINE_BUFFER_RVA, font_at, line_buffer};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::append_time::{append_time, show_hours};
use crate::text::draw_text::{Canvas, draw_text_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_section_best_times",
    rva(0x46762B),
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

const TABLES_RVA: usize = rva(0x742C98); // off_742C98: six tables
const SECTIONS: u32 = 6;
const SECTION_RVA: usize = rva(0x75595C); // section
const ROW_RVA: usize = rva(0x7559AC); // seconds
const ENTRY_LEN: u32 = 0x9C;
const LINE_LEN: usize = 0x100;

fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn line(bytes: &[u8]) -> Result<*mut u8, Refused> {
    if bytes.len() + 1 > LINE_LEN {
        return Err(BadInput("a line past the line buffer").into());
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len());
        line_buffer().add(bytes.len()).write(0);
    }
    Ok(line_buffer())
}

/// The Rust version.
pub unsafe fn draw_section_best_times(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let section: u32 = game_read(SECTION_RVA);
        if section >= SECTIONS {
            return Err(BadInput("a section past the six best-times tables").into());
        }
        let mut e: u32 = game_read(TABLES_RVA + 4 * section as usize);
        if e == 0 {
            return Err(BadInput("a section with no best-times table").into());
        }
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        let mut y = 0xF0i32;
        for n in 1..=10u32 {
            game_write(ROW_RVA, n);
            let (font, place_font) = if get(e + 0x40) == 1 {
                (0x51E524, 0x51E544)
            } else {
                (0x51E584, 0x51E5A4)
            };
            y = draw_text_at(font_at(font), (e + 0x20) as *const u8, 0xF0, y, canvas)?.y;
            let time = line(&append_time(get(e + 0x18), show_hours()))?;
            y = draw_text_at(font_at(font), time, 0x1E0, y, canvas)?.y;
            let place = line(&crate::strings::append_u32::append_u32(n))?;
            let o = draw_text_at(font_at(place_font), place, 0xC8, y, canvas)?;
            y = o.y;
            let palette = o.text as u32;
            let blit = |canvas: &mut dyn Canvas, glyph: u32, x: i32| {
                canvas.blit_glyph(BlitCall::new(glyph, x, y, palette, sprites))
            };
            blit(canvas, get(e + 8).wrapping_add(0x1C), 0xD2)?;
            let swapped = |v: u32| if (0x18..=0x19).contains(&v) { v ^ 1 } else { v };
            let flag = swapped(get(e + 0x10)).wrapping_add(0x44);
            blit(canvas, flag, 0x276)?;
            blit(canvas, swapped(flag).wrapping_add(0x44), 0x276)?;
            y = y.wrapping_add(0x19);
            e += ENTRY_LEN;
        }
    }
    Ok(())
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_section_best_times(canvas) }?;
    Ok(Left::nothing())
}

fn written() -> Vec<Region> {
    vec![(game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN), at(ROW_RVA)]
}

fn state() -> Vec<u8> {
    capture(&written())
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
    let inputs = [at(SECTION_RVA)];
    let saved = (capture(&inputs), state());
    for section in 0..SECTIONS {
        check_draw(
            &mut t,
            &format!("section {section}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(SECTION_RVA, section) },
            &rust,
            &state,
        );
    }
    restore(&inputs, &saved.0);
    restore(&written(), &saved.1);
    t.finish()
}
