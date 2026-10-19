//! `screens::draw_class_ranking` (0x47C16A): a page of the times of one class
//! (`dword_7558F4`) from the list at `dword_7556EC` (entries of 0x9C bytes, -1 ends it;
//! +4 the driver's kind or player, +8 the nationality, +0xC the name (a string id when +4
//! is 0, else a text), +0x10 the flag, +0x18 the time, +0x1C the class), ten rows from the
//! place `dword_7556B4` on.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! `g_firstPlayerTime` the class's first time (`menu::find_class_time`, ebx 1). The rows
//! before the place are passed over (counting the class's entries from 1 until the place).
//! Each row (`seconds` its place, and one past the last when the list ends after it, y from 0xF0, 0x19 apart, each draw's y handed on as edx
//! is): the name at x 0xFA (font 0x51E584; a string for kind 0, else the text; the player
//! `dword_7556C0` in 0x51E524 with the fonts below changed to theirs), the time at 0x258
//! (`draw_time_fixed_pitch`, 0x51E564 / 0x51E504), the place at 0xBE (0x51E5A4 / 0x51E544)
//! and below the first the gap (`glyph`): string 0x579 at 0x276 and the time at 0x2E4;
//! then `dword_7559B0` = y + 4, the nationality's sprite (0x1C + +8) at x 0xD2 and the
//! flag's (`+0x10` with 0x18 and 0x19 swapped, + 0x44) at 0x1AE, both from `g_spriteTable`
//! with ebx as the draws leave it as the palette.
//!
//! In the draw list 0x758E46 (the class times screen; a reading). Compared in shadow mode
//! (`dispatch_draw`; the line buffer and the four variables as state). Refuses (before
//! anything) a list without its end in 0x400 entries, a page that starts past the list
//! (the original reads past its end), a row drawn just before the end mark (the original
//! then leaves with a dword still pushed: its `popa` and `retn` take the wrong ones, the
//! original's slip), a name past the line buffer. Self-test: lists of the test's, places
//! 1 and 2, the player in them or not, nothing drawn.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::menu::find_class_time::{ENTRY_LEN, find_class_time};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::{LINE_BUFFER_RVA, font_at, line_buffer};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_class_ranking",
    rva(0x47C16A),
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

const LIST_RVA: usize = rva(0x7556EC); // dword_7556EC
const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const FIRST_PLACE_RVA: usize = rva(0x7556B4); // dword_7556B4
const PLAYER_RVA: usize = rva(0x7556C0); // dword_7556C0
const FIRST_TIME_RVA: usize = rva(0x7559BC); // g_firstPlayerTime
const PLACE_RVA: usize = rva(0x7559AC); // seconds
const GAP_RVA: usize = rva(0x7559C0); // glyph
const SPRITE_Y_RVA: usize = rva(0x7559B0); // dword_7559B0
const LIMIT: u32 = 0x400;
const LINE_LEN: usize = 0x100;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// Each row's entry and place, and the place left after the last.
type Rows = (Vec<(u32, u32)>, Option<u32>);

/// The rows to draw (each entry with its place), worked out before anything is drawn, and
/// the place `seconds` is left at when the list ends while the next row is looked for (the
/// loop writes it before it looks).
fn rows() -> Result<Rows, Refused> {
    let (list, class, first) = (g(LIST_RVA), g(CLASS_RVA), g(FIRST_PLACE_RVA));
    let mut e = list;
    let mut seen = 0u32;
    let mut steps = 0;
    let mut step = |e: &mut u32| -> Result<bool, Refused> {
        steps += 1;
        if steps > LIMIT {
            return Err(BadInput("a list without its end in 0x400 entries").into());
        }
        *e += ENTRY_LEN;
        Ok(get(*e) == u32::MAX)
    };
    loop {
        if get(e + 0x1C) == class {
            seen = seen.wrapping_add(1);
            if seen == first {
                break;
            }
        }
        if step(&mut e)? {
            return Err(
                BadInput("a page that starts past the list (the original reads on)").into(),
            );
        }
    }
    let mut out = Vec::new();
    let mut place = first;
    loop {
        while get(e + 0x1C) != class {
            if step(&mut e)? {
                return Ok((out, Some(place)));
            }
        }
        out.push((e, place));
        if step(&mut e)? {
            return Err(
                BadInput("a row just before the end mark (the original's stack slip)").into(),
            );
        }
        place = place.wrapping_add(1);
        if place == first.wrapping_add(10) {
            return Ok((out, None));
        }
    }
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
pub unsafe fn draw_class_ranking(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::strings::append_time::{append_time, show_hours};
    use crate::text::draw_time_fixed_pitch::draw_time_fixed_pitch_at;
    let (rows, last_place) = rows()?;
    let first_time = find_class_time(g(LIST_RVA), g(CLASS_RVA), true)?;
    unsafe {
        game_write(FIRST_TIME_RVA, first_time);
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        let mut y = 0xF0i32;
        for (k, &(e, place)) in rows.iter().enumerate() {
            if k > 0 {
                y = y.wrapping_add(0x19);
            }
            game_write(PLACE_RVA, place);
            let kind = get(e + 4);
            let mine = kind != 0 && kind == g(PLAYER_RVA);
            let (name_font, time_font, place_font) = if mine {
                (0x51E524, 0x51E504, 0x51E544)
            } else {
                (0x51E584, 0x51E564, 0x51E5A4)
            };
            let o = if kind == 0 {
                draw_string_by_id_at(get(e + 0xC), font_at(name_font), 0xFA, y, canvas)?
            } else {
                draw_text_at(
                    font_at(name_font),
                    get(e + 0xC) as *const u8,
                    0xFA,
                    y,
                    canvas,
                )?
            };
            y = o.y;
            let time = get(e + 0x18);
            draw_time_fixed_pitch_at(
                font_at(time_font),
                line(&append_time(time, show_hours()))?,
                0x258,
                y,
                canvas,
            )?;
            let o = draw_text_at(
                font_at(place_font),
                line(&crate::strings::append_u32::append_u32(place))?,
                0xBE,
                y,
                canvas,
            )?;
            y = o.y;
            let mut palette = o.text as u32;
            if place != 1 {
                let gap = time.wrapping_sub(first_time);
                game_write(GAP_RVA, gap);
                y = draw_string_by_id_at(0x579, font_at(place_font), 0x276, y, canvas)?.y;
                draw_time_fixed_pitch_at(
                    font_at(time_font),
                    line(&append_time(gap, show_hours()))?,
                    0x2E4,
                    y,
                    canvas,
                )?;
                palette = line_buffer() as u32;
            }
            let sprite_y = y.wrapping_add(4);
            game_write(SPRITE_Y_RVA, sprite_y);
            canvas.blit_glyph(BlitCall::new(
                get(e + 8).wrapping_add(0x1C),
                0xD2,
                sprite_y,
                palette,
                sprites,
            ))?;
            let flag = get(e + 0x10);
            let flag = if (0x18..=0x19).contains(&flag) {
                flag ^ 1
            } else {
                flag
            };
            canvas.blit_glyph(BlitCall::new(
                flag.wrapping_add(0x44),
                0x1AE,
                sprite_y,
                palette,
                sprites,
            ))?;
        }
        if let Some(place) = last_place {
            game_write(PLACE_RVA, place);
        }
    }
    Ok(())
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_class_ranking(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN),
        at(FIRST_TIME_RVA),
        at(PLACE_RVA),
        at(GAP_RVA),
        at(SPRITE_Y_RVA),
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
    let inputs: [Region; 4] = [
        at(LIST_RVA),
        at(CLASS_RVA),
        at(FIRST_PLACE_RVA),
        at(PLAYER_RVA),
    ];
    let saved = (capture(&inputs), state());
    let names: Vec<Vec<u8>> = (0..16)
        .map(|n| format!("Driver {n}\0").into_bytes())
        .collect();
    // Sixteen entries, classes 2, 3, 2, 2, 3, ...; the last two of another class, so that
    // no drawn row is just before the end mark.
    let mut list = vec![0u32; 17 * ENTRY_LEN as usize / 4];
    let base = list.as_mut_ptr() as u32;
    for n in 0..16u32 {
        let e = base + n * ENTRY_LEN;
        let put = |o: u32, v: u32| unsafe { ((e + o) as *mut u32).write_unaligned(v) };
        let kind = [0u32, 1, 5, 0, 2][n as usize % 5];
        put(0, n);
        put(4, kind);
        put(8, n % 7);
        put(
            0xC,
            if kind == 0 {
                0x249 + n
            } else {
                names[n as usize].as_ptr() as u32
            },
        );
        put(0x10, [0x18u32, 0x19, 3, 0x20][n as usize % 4]);
        put(0x18, 6000 + 137 * n);
        put(
            0x1C,
            if n >= 14 {
                9
            } else {
                [2u32, 3, 2][n as usize % 3]
            },
        );
    }
    unsafe { ((base + 16 * ENTRY_LEN) as *mut u32).write_unaligned(u32::MAX) };
    for (class, first, player) in [(2u32, 1u32, 1u32), (3, 1, 5), (2, 2, 2), (2, 3, 7)] {
        check_draw(
            &mut t,
            &format!("class {class}, from {first}, player {player}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(LIST_RVA, base);
                game_write(CLASS_RVA, class);
                game_write(FIRST_PLACE_RVA, first);
                game_write(PLAYER_RVA, player);
            },
            &rust,
            &state,
        );
    }
    restore(&inputs, &saved.0);
    restore(
        &[
            (game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN),
            at(FIRST_TIME_RVA),
            at(PLACE_RVA),
            at(GAP_RVA),
            at(SPRITE_Y_RVA),
        ],
        &saved.1,
    );
    drop(list);
    t.finish()
}
