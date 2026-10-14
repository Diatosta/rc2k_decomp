//! `screens::draw_inspection_status` (0x4A6B54): works out the inspection status, a percentage,
//! from the current car's thirteen damage values and draws it as a bar at the bottom of the
//! service area, with a sprite to its left and a label at each end.
//!
//! In:  ebx, passed on as the palette of the sprite at the bar's left
//! Out: what the last `draw_string_by_id` leaves (carry, eax = 0, ebx, ecx = 652, edx);
//!      edi = the player; esi = the address of the repair flag of the last damaged part
//!      looked at, or unchanged; ebp preserved
//! The percentage is 100 minus the weights of the parts that are damaged and not chosen for
//! repair. For menu items 2, 3, 4, 5 and 7 .. 15: the part counts if the car's value at
//! +0xCC + 4 * item is at least `dword_75ABD0[item]` and the item's repair flag is 0; its
//! weight is `dword_75AC40[item]`. The result is left in `dword_75AC80`.
//! The items are the fourteen parts of the repair section without the turbo (item 6).
//!
//! Screen: seen in a screenshot of the championship's service area (2026-10-10): "ESTADO DE INSPECÇÃO" with a
//! red to green bar between "REPROV." (string 0x4AE) and "PASSAR" (0x4AF): fail .. pass. The
//! "ESTADO DE INSPECÇÃO" label itself is drawn by `draw_service_area`. Named
//! `draw_car_condition` before the screenshot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EDI, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::gfx::blit_bar::{Bar, blit_bar};
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::blit_marker_pair::sprite_table;
use crate::screens::font_is_loaded;
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable, push_dwords};
use crate::screens::stage_panel::label;
use crate::service::state::{current_player, repair_flag, repair_flag_addr};
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::Canvas;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_inspection_status",
    rva(0x4A6B54),
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

const THRESHOLDS_RVA: usize = rva(0x75ABD0); // dword_75ABD0[item] (items 2 .. 15 are used)
const WEIGHTS_RVA: usize = rva(0x75AC40); // dword_75AC40[item]
const CONDITION_RVA: usize = rva(0x75AC80); // dword_75AC80
const WORK_RVA: usize = rva(0x7559EC); // dword_7559EC: a work variable, here the weights' sum
const CAR_DAMAGE_OFFSET: u32 = 0xCC; // car +0xCC + 4 * item

/// The menu items whose parts count.
const PARTS: [u32; 13] = [2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15];

const FONT: usize = 0x51E3C4;
const LEFT_LABEL: u32 = 0x4AE;
const RIGHT_LABEL: u32 = 0x4AF;
const BAR: Bar = Bar {
    x: 0x19D,
    y: 0x229,
    width: 0xD0,
    fill_glyph: 0x89,
    end_glyph: 0x87,
    percent: 0,
};
const ICON_GLYPH: u32 = 0x88;

unsafe fn table(table_rva: usize, item: u32) -> i32 {
    unsafe { ((game_addr(table_rva) as *const i32).add(item as usize)).read_unaligned() }
}

// ---- Rust version ----

/// The percentage and the item of the last damaged part looked at (its repair flag's address
/// is what the original leaves in esi).
pub unsafe fn inspection_status(player: u32) -> (i32, Option<u32>) {
    let mut lost = 0i32;
    let mut last_damaged = None;
    for item in PARTS {
        unsafe {
            let damage =
                (player.wrapping_add(CAR_DAMAGE_OFFSET + 4 * item) as *const i32).read_unaligned();
            if damage < table(THRESHOLDS_RVA, item) {
                continue;
            }
            last_damaged = Some(item);
            if repair_flag(item) == 0 {
                lost = lost.wrapping_add(table(WEIGHTS_RVA, item));
            }
        }
    }
    (100i32.wrapping_sub(lost), last_damaged)
}

/// `icon_palette` is what the caller has in ebx. Refuses a missing player.
pub unsafe fn draw_inspection_status(
    icon_palette: u32,
    canvas: &mut dyn Canvas,
) -> Result<Left, Refused> {
    unsafe {
        let Some(player) = current_player() else {
            return Err(
                BadInput("no current player object: the original reads through null").into(),
            );
        };
        let (condition, last_damaged) = inspection_status(player);
        game_write(WORK_RVA, 100i32.wrapping_sub(condition));
        game_write(CONDITION_RVA, condition);

        Bar {
            percent: condition,
            ..BAR
        }
        .write();
        blit_bar(&mut |call| canvas.blit_glyph(call))?;
        canvas.blit_glyph(BlitCall::new(
            ICON_GLYPH,
            0x161,
            0x228,
            icon_palette,
            sprite_table(),
        ))?;
        label(canvas, LEFT_LABEL, FONT, 0x17F, 0x22C)?;
        let out = draw_string_by_id_at(
            RIGHT_LABEL,
            crate::screens::font_at(FONT),
            0x28C,
            0x22C,
            canvas,
        )?;

        let mut left = Left::of_text(&out).with(EDI, player);
        if let Some(item) = last_damaged {
            left = left.with(ESI, repair_flag_addr(item));
        }
        Ok(left)
    }
}

// ---- hook ----

fn state() -> Vec<u8> {
    let mut s = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
    push_dwords(
        &mut s,
        &[
            CONDITION_RVA,
            WORK_RVA,
            rva(0x7559AC),
            rva(0x755224),
            rva(0x755228),
            rva(0x75522C),
            rva(0x755230),
            rva(0x755234),
            rva(0x755238),
        ],
    );
    s
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_inspection_status(r.ebx, canvas) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test: the current car, with the repair flags all clear, all set and mixed ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if blob().is_null()
        || !font_is_loaded(FONT)
        || !is_table_id(LEFT_LABEL)
        || !is_table_id(RIGHT_LABEL)
    {
        return t.skip("the string table or the font is not loaded");
    }
    if current_player().is_none() {
        return t.skip("there is no current player object");
    }
    let saved_bar = Bar::read();
    let saved_flags: Vec<u32> = PARTS.iter().map(|&item| repair_flag(item)).collect();
    let saved_text = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
    let saved_work: [u32; 3] = unsafe {
        [
            game_read(CONDITION_RVA),
            game_read(WORK_RVA),
            game_read(rva(0x7559AC)),
        ]
    };
    let set_flags = |flags: &dyn Fn(usize) -> u32| {
        for (i, &item) in PARTS.iter().enumerate() {
            unsafe { (repair_flag_addr(item) as *mut u32).write_unaligned(flags(i)) };
        }
    };

    let patterns: [(&str, &dyn Fn(usize) -> u32); 4] = [
        ("as found", &|i| saved_flags[i]),
        ("no repair chosen", &|_| 0),
        ("every repair chosen", &|_| 1),
        ("every other repair chosen", &|i| (i % 2) as u32),
    ];
    for (what, flags) in patterns {
        let prepare = || unsafe {
            set_flags(flags);
            game_write(CONDITION_RVA, 0x5EED_0001u32);
            game_write(WORK_RVA, 0x5EED_0002u32);
            std::ptr::copy_nonoverlapping(saved_text.as_ptr(), text(), TEXT_LEN);
        };
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

    set_flags(&|i| saved_flags[i]);
    saved_bar.write();
    unsafe {
        game_write(CONDITION_RVA, saved_work[0]);
        game_write(WORK_RVA, saved_work[1]);
        game_write(rva(0x7559AC), saved_work[2]);
        std::ptr::copy_nonoverlapping(saved_text.as_ptr(), text(), TEXT_LEN);
    }
    t.finish()
}
