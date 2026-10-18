//! `screens::draw_graphics_settings` (0x46FDF0): the graphics page: item 2 choosable only
//! when `dword_4F50D4` + `dword_4F50D8` is 2; the settings' names (strings by the work
//! values, each kept in `dword_7559EC`) and the screen mode "w x h x bits" (`unk_4F50E8`
//! [`dword_4F50C8`], built in the shared text buffer) at x 0x1D1 from y 0xDC every 0x1B, in
//! the font at 0x51E8C4.
//!
//! In:  nothing
//! Out: as the last `draw_string_by_id` leaves them (carry, eax 0, ebx, ecx, edx)
//!
//! In a menu loop's draw list (0x758CB2). Refuses what `menu::enable_item` refuses (no
//! current menu), before anything. Compared in shadow mode (`dispatch_draw`; the item and
//! `dword_7559EC` as state). Self-test: the page's work values and the switches varied,
//! nothing drawn (skipped without the fonts or a menu).

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::menu::disable_item::{disable_item, item_region};
use crate::menu::enable_item::enable_item;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::font_at;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_graphics_settings",
    rva(0x46FDF0),
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

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

/// (the setting, its first string, its row) for the names; row 1 is the screen mode.
const NAMES: [(u32, u32, i32); 8] = [
    (0x4F50C4, 0x38F, 0),
    (0x4F50CC, 0x391, 2),
    (0x4F50D0, 0x56D, 3),
    (0x755650, 0x56D, 4),
    (0x75565C, 0x56D, 5),
    (0x755660, 0x396, 6),
    (0x755664, 0x393, 7),
    (0x755668, 0x394, 8),
];

/// The Rust version: the registers the last name leaves.
pub unsafe fn draw_graphics_settings(canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    use crate::strings::format_u32::format_u32;
    use crate::text::draw_text::draw_text_at;
    unsafe {
        if k(0x4F50D4).wrapping_add(k(0x4F50D8)) == 2 {
            enable_item(2)?;
        } else {
            disable_item(2)?;
        }
        let mut left = Left::nothing();
        let font = font_at(0x51E8C4);
        for (n, (setting, first, row)) in NAMES.into_iter().enumerate() {
            let id = k(setting).wrapping_add(first);
            game_write(rva(0x7559EC), id);
            let out = draw_string_by_id_at(id, font, 0x1D1, 0xDC + 0x1B * row, canvas)?;
            left = Left::carry(out.too_wide)
                .with(EAX, 0)
                .with(EBX, out.text as u32)
                .with(ECX, out.x as u32)
                .with(EDX, out.y as u32);
            if n == 0 {
                let mode = game_addr(rva(0x4F50E8)) as u32 + 0x10 * k(0x4F50C8);
                let v = |i: u32| ((mode + 4 * i) as *const u32).read_unaligned();
                let mut line = format_u32(v(0), 0);
                line.push(b'x');
                line.extend(format_u32(v(1), 0));
                line.push(b'x');
                line.extend(format_u32(v(2), 0));
                line.push(0);
                std::ptr::copy_nonoverlapping(line.as_ptr(), text(), line.len());
                draw_text_at(font, text(), 0x1D1, 0xF7, canvas)?;
            }
        }
        Ok(left)
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_graphics_settings(canvas) }
}

fn state() -> Vec<u8> {
    let mut regions: Vec<Region> = vec![(text() as u32, TEXT_LEN), at(rva(0x7559EC))];
    regions.extend(item_region(2));
    capture(&regions)
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
    // Only looked up: enabling it here, before the capture, left the current menu changed
    // (the stage selection's items, 2026-10-18).
    if crate::menu::item_at::item_at(2).is_err() {
        return t.skip("no current menu");
    }
    let mut kept: Vec<Region> = vec![
        (game_addr(rva(0x4F50C4)) as u32, 0x18),
        (game_addr(rva(0x755650)) as u32, 0x1C),
        at(rva(0x7559EC)),
    ];
    kept.extend(item_region(2));
    let saved = capture(&kept);
    for (seed, switches) in [(0u32, [1u32, 1]), (1, [0, 2]), (2, [2, 0])] {
        check_draw(
            &mut t,
            &format!("seed {seed} switches {switches:?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x4F50C4), [seed & 1, seed % 3, seed & 1, seed & 1]);
                game_write(rva(0x4F50D4), switches);
                game_write(rva(0x755650), seed & 1);
            },
            &rust,
            &state,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
