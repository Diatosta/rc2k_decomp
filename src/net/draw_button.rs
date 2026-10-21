//! `net::draw_button` (0x416CE0): a button of the network screens: string eax measured
//! (`text::measure_string_by_id` with `palette`), its frame drawn from the menu sprites
//! (`g_spriteTable` glyphs 0x11, 0x12 once per 32 pixels of the width, at least once, and
//! 0x13: `gfx::blit_glyph`), the string drawn on it (`text::draw_string_by_id`) in
//! `unk_51E704`, or `unk_51E644` when it is the highlighted item (`g_listChoice`) and either
//! being edited (`dword_52A4B8`) or in the blink's lit half (bit 3 of `dword_51D41F` -
//! `dword_51D423` clear), or `stru_51E7C4` with flag 2.
//!
//! In:  eax = the string, ebx = the flags (1: the frame centred on x, else ending half its
//!      width left of x and the text centred too; 2: the dim font), ecx = x, edx = y,
//!      esi = the item
//! Out: every register kept (`pusha` .. `popa`)
//! The arguments are kept in `dword_5AB490` .., the width in `dword_5AB4A0`, the frame's
//! tiles in `dword_5AB4A4`. The blits get ebx as the measuring and the blits before leave
//! it (the sprites are drawn as they are). Callers: `net::setting_label`,
//! `net::lobby_button`. Its calls through `part` (`net::lobby_calls`: the Rust versions).
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins (the width given), against the Rust version: each flag, widths of 0, 20, 64,
//! 65, the item highlighted and blinking, edited, another.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    BLIT, CHOICE, DRAW_STRING, EDITED, FRAME_CLOCK, MEASURE_STRING, MOVE_CLOCK, PALETTE, Part,
    SPRITES, Saved, a, answer, dispatch_lobby, dword, g, p, region, run_both,
};
use rc2k_core::net::screens::{self, ButtonFont};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::draw_button", rva(0x416CE0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ARGS: usize = 0x5AB490; // dword_5AB490: the string, flags, x, y; +0x18 the item
const WIDTH: usize = 0x5AB4A0; // dword_5AB4A0
const TILES: usize = 0x5AB4A4; // dword_5AB4A4
const ITEM: usize = 0x5AB4A8; // dword_5AB4A8
pub const FONT_PLAIN: usize = 0x51E704; // unk_51E704
pub const FONT_LIT: usize = 0x51E644; // unk_51E644
const FONT_DIM: usize = 0x51E7C4; // stru_51E7C4

/// The font of item `item`'s text: lit when it is the highlighted item and being edited
/// or in the blink's lit half.
pub fn item_font(item: u32, plain: usize, lit: usize) -> u32 {
    // Converted: `rc2k_core::net::screens::{item_lit, blink_lit}`.
    let blink = || screens::blink_lit(g(FRAME_CLOCK), g(MOVE_CLOCK));
    if screens::item_lit(item, g(CHOICE), || g(EDITED), blink) {
        a(lit)
    } else {
        a(plain)
    }
}

/// The Rust version: the registers it leaves (the entry's).
pub fn draw_button(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(ARGS, r.eax);
    p(ARGS + 4, r.ebx);
    p(ARGS + 8, r.ecx);
    p(ARGS + 12, r.edx);
    p(ITEM, r.esi);
    r.ebx = a(PALETTE);
    part(MEASURE_STRING, &mut r)?;
    p(WIDTH, r.ecx);
    // Converted: `rc2k_core::net::screens` (`button`, `item_lit`, `button_font`).
    let flags = g(ARGS + 4);
    let place = screens::button(g(WIDTH), g(ARGS + 8), flags);
    p(TILES, place.tiles);
    if flags & screens::CENTRED != 0 {
        p(ARGS + 8, place.text_x);
    }
    r.edi = place.left;
    let blit = |r: &mut Regs, glyph: u32, part: Part| -> Result<(), Refused> {
        r.eax = glyph;
        r.ecx = r.edi;
        r.edx = g(ARGS + 12);
        r.esi = g(SPRITES);
        part(BLIT, r)
    };
    blit(&mut r, screens::LEFT_END, part)?;
    r.edi = r.edi.wrapping_add(screens::END_WIDTH);
    let kept = r.edi;
    r.ebp = g(TILES);
    loop {
        blit(&mut r, screens::TILE, part)?;
        r.edi = r.edi.wrapping_add(screens::TILE_WIDTH);
        r.ebp = r.ebp.wrapping_sub(1);
        if r.ebp == 0 {
            break;
        }
    }
    r.edi = kept.wrapping_add(g(WIDTH));
    blit(&mut r, screens::RIGHT_END, part)?;
    r.edi = r.edi.wrapping_add(screens::END_WIDTH);
    r.eax = g(ARGS);
    r.ecx = g(ARGS + 8);
    r.edx = g(ARGS + 12);
    r.esi = g(ITEM);
    let lit = screens::item_lit(
        r.esi,
        g(CHOICE),
        || g(EDITED),
        || {
            // The blink's clock is left in edi.
            let (frame, moved) = (g(FRAME_CLOCK), g(MOVE_CLOCK));
            r.edi = frame.wrapping_sub(moved);
            screens::blink_lit(frame, moved)
        },
    );
    r.ebx = a(match screens::button_font(flags, lit) {
        ButtonFont::Plain => FONT_PLAIN,
        ButtonFont::Lit => FONT_LIT,
        ButtonFont::Dim => FONT_DIM,
    });
    part(DRAW_STRING, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, draw_button);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov dword_5AB490, eax
    let copy = match crate::net::lobby::copy(
        0x416CE0,
        0x13A,
        &[0x60, 0xA3, 0x90, 0xB4, 0x5A, 0x00],
        &[0x24, 0x8B, 0xAD, 0xD2, 0x133],
        &[MEASURE_STRING, BLIT, DRAW_STRING],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        region(ARGS, 0x1C),
        dword(CHOICE),
        dword(EDITED),
        dword(FRAME_CLOCK),
        dword(MOVE_CLOCK),
    ];
    let saved = Saved::new(&all);
    // (flags, the width, the highlighted item, edited, the blink clock)
    let cases: [(u32, u32, u32, u32, u32); 9] = [
        (0, 0x40, 1, u32::MAX, 0),
        (1, 0x40, 1, u32::MAX, 8),
        (2, 0, 2, u32::MAX, 0),
        (3, 20, 1, 1, 8),
        (0, 0x41, 3, u32::MAX, 0),
        (1, 0xFFFF_FFF0, 1, u32::MAX, 0x10),
        (0, 0x100, 1, 2, 8),
        (0, 7, 0, u32::MAX, 0),
        (1, 0x5D, 1, 1, 0),
    ];
    for (flags, width, choice, edited, clock) in cases {
        run_both(
            &mut t,
            &format!(
                "flags {flags}, width {width:#x}, highlighted {choice}, edited {edited:#x}, clock {clock:#x}"
            ),
            copy.addr(),
            Regs {
                eax: 0x15D,
                ebx: flags,
                ecx: 0x310,
                edx: 0x223,
                esi: 1,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                p(CHOICE, choice);
                p(EDITED, edited);
                p(FRAME_CLOCK, 0x1000 + clock);
                p(MOVE_CLOCK, 0x1000);
                answer(Box::new(move |place, r| match place {
                    MEASURE_STRING => {
                        (r.eax, r.ebx, r.ecx, r.edx) = (0, 1, width, 0x40);
                    }
                    BLIT => {
                        r.ebx = r.ecx ^ 0x77;
                        r.eax = 0x5A;
                    }
                    _ => {}
                }));
            },
            &draw_button,
        );
    }
    drop(saved);
    t.finish()
}
