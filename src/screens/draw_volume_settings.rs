//! `screens::draw_volume_settings` (0x476A92): the sound page's volumes: the setting's name
//! (string 0x56D + `dword_5180F4`, kept in `seconds`) at (0x1EA, 0x1B3), and eight rows from
//! y 0xDA every 0x1C: a back sprite 0x19 at x 0x222, the bar (`gfx::blit_bar` at x 0x221,
//! 0x76 wide, sprites 0x18 / 0x87, the volume `g_masterVolumeShown` / `dword_755688` ..
//! `dword_7556A0` as 0x40ths: percent v * 100 / 0x40), the knob sprite 0x1B at (0x1E6, y - 1)
//! and strings 0x547 / 0x548 at x 0x203 / 0x2B7, y + 3, in the font at 0x51E3C4.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758C8E). A volume whose v * 100 wraps to 0 is the
//! original's fatal 0x80701010 (`loc_4D9612`), raised from Rust where the original does. The
//! sprites' palettes are ebx as the original leaves it. Compared in shadow mode
//! (`dispatch_draw`; `seconds` and the bar's globals as state). Self-test: the volumes as
//! they are, 0, 1, 0x20, 0x40, and the name's switch, nothing drawn.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_bar::{BAR_HAS_NO_LENGTH, Bar, blit_bar};
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::font_at;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, capture, restore};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_volume_settings",
    rva(0x476A92),
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

const VOLUMES_RVA: usize = rva(0x755684); // g_masterVolumeShown, then dword_755688 .. dword_7556A0
const NAME_SWITCH_RVA: usize = rva(0x5180F4); // dword_5180F4
const SECONDS_RVA: usize = rva(0x7559AC); // seconds

pub unsafe fn draw_volume_settings(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let id = game_read::<u32>(NAME_SWITCH_RVA).wrapping_add(0x56D);
        game_write(SECONDS_RVA, id);
        let mut palette =
            draw_string_by_id_at(id, font_at(0x51E8C4), 0x1EA, 0x1B3, canvas)?.text as u32;
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        for n in 0..8 {
            canvas.blit_glyph(BlitCall::new(
                0x19,
                0x222,
                0xDA + 0x1C * n,
                palette,
                sprites,
            ))?;
        }
        for n in 0..8 {
            let v: u32 = game_read(VOLUMES_RVA + 4 * n as usize);
            let percent = if v == 0 {
                0
            } else {
                let p = v.wrapping_mul(100);
                if p == 0 {
                    return Err(BAR_HAS_NO_LENGTH.into());
                }
                p / 0x40
            };
            Bar {
                x: 0x221,
                y: 0xDA + 0x1C * n,
                width: 0x76,
                fill_glyph: 0x18,
                end_glyph: 0x87,
                percent: percent as i32,
            }
            .write();
            blit_bar(&mut |call| canvas.blit_glyph(call))?;
        }
        for n in 0..8 {
            let y = 0xD9 + 0x1C * n;
            canvas.blit_glyph(BlitCall::new(0x1B, 0x1E6, y, palette, sprites))?;
            draw_string_by_id_at(0x547, font_at(0x51E3C4), 0x203, y + 4, canvas)?;
            palette =
                draw_string_by_id_at(0x548, font_at(0x51E3C4), 0x2B7, y + 4, canvas)?.text as u32;
        }
        Ok(())
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_volume_settings(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    capture(&[
        g(0x7559AC, 4),
        g(0x755224, 0x18),
        (crate::strings::text_from_ids::text() as u32, 0x100),
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
    let regions: Vec<Region> = vec![
        (game_addr(VOLUMES_RVA) as u32, 0x20),
        (game_addr(NAME_SWITCH_RVA) as u32, 4),
        (game_addr(rva(0x7559AC)) as u32, 4),
        (game_addr(rva(0x755224)) as u32, 0x18),
    ];
    let saved = capture(&regions);
    let switch: u32 = unsafe { game_read(NAME_SWITCH_RVA) };
    let sets: [Option<[u32; 8]>; 3] = [
        None,
        Some([0, 1, 0x20, 0x40, 0x3F, 2, 0x10, 0x30]),
        Some([0x40; 8]),
    ];
    for (s, set) in sets.iter().enumerate() {
        for name in [switch, 0, 1] {
            check_draw(
                &mut t,
                &format!("volumes {s}, name switch {name}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    game_write(NAME_SWITCH_RVA, name);
                    if let Some(v) = set {
                        for (n, x) in v.iter().enumerate() {
                            game_write(VOLUMES_RVA + 4 * n, *x);
                        }
                    }
                },
                &rust,
                &state,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
