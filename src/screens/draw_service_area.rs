//! `screens::draw_service_area` (0x4A5FA2): draws everything on the service area screen that
//! is text, bars and small sprites, over the picture: a title with the car's number, the
//! service time readouts, fourteen bars from the car's values at +0xD4 .. +0x108 with their
//! labels and sprites, the setup bars' labels and markers, the repair times and the car
//! condition bar.
//!
//! In:  nothing
//! Out: every register preserved; carry as `draw_inspection_status` leaves it
//! The order is the original's: the title ("<string 0x190> <car index + 1> <string 0x486>",
//! built in the shared text buffer), `draw_service_time`, the `STEPS` list, the string
//! 0x4B0 + `dword_75AC9C`, `draw_repair_times`, the six setup markers, `draw_inspection_status`.
//! The sprites in the list are blitted with whatever ebx holds as the palette, which is the
//! end of the text drawn just before them; reproduced. It also sets `dword_6218AC` to 0.
//! Its address is in the service area's list at 0x75924A; it is also called from 0x46D001.
//! What is on it, from the owner's screenshot of 2026-10-10 (Portuguese game): the title
//! "JOGADOR 1 ÁREA DE SERVIÇO"; a repair section with fourteen parts in two columns (engine,
//! exhaust, turbo, cooling, clutch, differential, gearbox; body, electronics, lights,
//! steering, suspension, brakes, tyres), each with an icon, a repair time and a damage bar
//! (car +0xD4 .. +0x108, in that row order left then right); "n/d" on the turbo's bar when
//! the car's dword at +0x90 is not 0; five gear ratio pictures (acceleration .. top speed)
//! and six tyre pictures with the name of the chosen tyre (`dword_75AC9C`); a setup section
//! with four sliders; and the inspection status bar. The four big buttons at the bottom
//! (load setup, save setup, view stage, continue) are labels drawn here too.
//!
//! Screen: confirmed by the owner on 2026-10-10 with `blank on screens::draw_service_area`:
//! the service area of a championship. All text and icons disappeared; the boxes that hold
//! them stayed, so those are drawn by something else (not ported).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_bar::{Bar, blit_bar};
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::blit_marker_pair::sprite_table;
use crate::screens::draw_inspection_status::draw_inspection_status;
use crate::screens::draw_repair_times::draw_repair_times;
use crate::screens::draw_service_time::draw_service_time;
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable, push_dwords};
use crate::screens::setup_markers::draw_marker;
use crate::screens::{
    draw_brake_balance_marker, draw_ride_height_marker, draw_ride_height_previous_marker,
    draw_steering_sensitivity_marker, draw_suspension_stiffness_marker,
    draw_suspension_stiffness_previous_marker, font_at, font_is_loaded, line_buffer, set_line,
};
use crate::service::state::{CAR_INDEX_RVA, current_car};
use crate::strings::append_string_by_id::push_string;
use crate::strings::append_u32::append_u32;
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_service_area",
    rva(0x4A5FA2),
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

const CLEARED_RVA: usize = rva(0x6218AC); // dword_6218AC: set to 0 here, read nowhere known
const WORK_RVA: usize = rva(0x7559EC); // dword_7559EC: a work variable
const SETTING_RVA: usize = rva(0x75AC9C); // dword_75AC9C: the chosen tyre, 0 .. 5; its name is string 0x4B0 + this
const SETTING_FIRST_STRING: u32 = 0x4B0;

const TITLE_FONT: usize = 0x51E8A4;
const TITLE_BEFORE: u32 = 0x190;
const TITLE_AFTER: u32 = 0x486;
const SETTING_FONT: usize = 0x51E3C4;

const DAMAGE_BAR: Bar = Bar {
    x: 0,
    y: 0,
    width: 0xA0,
    fill_glyph: 0x86,
    end_glyph: 0x87,
    percent: 0,
};

enum Step {
    /// (string, font, x, y)
    Label(u32, usize, i32, i32),
    /// (offset in the car object, string, font, x, y): drawn only if that dword is not 0.
    LabelIf(u32, u32, usize, i32, i32),
    /// (x, y, offset in the car object of the percentage)
    DamageBar(i32, i32, u32),
    /// (glyph of the sprite table, x, y)
    Sprite(u32, i32, i32),
}
use Step::{DamageBar, Label, LabelIf, Sprite};

const STEPS: [Step; 83] = [
    Label(0x488, 0x51E6A4, 400, 68),
    DamageBar(236, 100, 0xD4),
    DamageBar(571, 100, 0xD8),
    DamageBar(236, 129, 0xDC),
    DamageBar(571, 129, 0xE0),
    DamageBar(236, 158, 0xE4),
    DamageBar(571, 158, 0xE8),
    DamageBar(236, 187, 0xEC),
    DamageBar(571, 187, 0xF0),
    DamageBar(236, 216, 0xF4),
    DamageBar(571, 216, 0xF8),
    DamageBar(236, 245, 0xFC),
    DamageBar(571, 245, 0x100),
    DamageBar(236, 274, 0x104),
    DamageBar(571, 274, 0x108),
    Label(0x492, 0x51E3E4, 106, 97),
    Label(0x499, 0x51E3E4, 441, 97),
    Label(0x493, 0x51E3E4, 106, 126),
    Label(0x49A, 0x51E3E4, 441, 126),
    Label(0x494, 0x51E3E4, 106, 155),
    Label(0x49B, 0x51E3E4, 441, 155),
    LabelIf(0x90, 0x578, 0x51E3C4, 319, 159),
    Label(0x495, 0x51E3E4, 106, 184),
    Label(0x49C, 0x51E3E4, 441, 184),
    Label(0x496, 0x51E3E4, 106, 213),
    Label(0x49D, 0x51E3E4, 441, 213),
    Label(0x497, 0x51E3E4, 106, 242),
    Label(0x49E, 0x51E3E4, 441, 242),
    Label(0x498, 0x51E3E4, 106, 271),
    Label(0x49F, 0x51E3E4, 441, 271),
    Label(0x4A0, 0x51E3E4, 70, 338),
    Label(0x4A1, 0x51E404, 397, 338),
    Label(0x487, 0x51E6A4, 400, 353),
    Label(0x4A2, 0x51E3E4, 106, 382),
    Label(0x4A5, 0x51E3E4, 441, 382),
    Label(0x4A8, 0x51E3E4, 106, 438),
    Label(0x4AB, 0x51E3E4, 441, 438),
    Sprite(0x68, 67, 97),
    Sprite(0x69, 402, 97),
    Sprite(0x6A, 67, 126),
    Sprite(0x6B, 402, 126),
    Sprite(0x6C, 67, 155),
    Sprite(0x6D, 402, 155),
    Sprite(0x6E, 67, 184),
    Sprite(0x6F, 402, 184),
    Sprite(0x70, 67, 213),
    Sprite(0x71, 402, 213),
    Sprite(0x72, 67, 242),
    Sprite(0x73, 402, 242),
    Sprite(0x74, 67, 271),
    Sprite(0x75, 402, 271),
    Sprite(0x7B, 70, 301),
    Sprite(0x7C, 136, 301),
    Sprite(0x7D, 202, 301),
    Sprite(0x7E, 268, 301),
    Sprite(0x7F, 334, 301),
    Sprite(0x80, 405, 301),
    Sprite(0x81, 460, 301),
    Sprite(0x82, 515, 301),
    Sprite(0x83, 570, 301),
    Sprite(0x84, 625, 301),
    Sprite(0x85, 680, 301),
    Sprite(0x6F, 67, 382),
    Sprite(0x73, 402, 382),
    Sprite(0x71, 67, 438),
    Sprite(0x71, 402, 438),
    Sprite(0x7A, 70, 412),
    Sprite(0x7A, 405, 412),
    Sprite(0x7A, 70, 468),
    Sprite(0x7A, 405, 468),
    Label(0x48F, 0x51E6E4, 340, 551),
    Label(0x489, 0x51E6A4, 233, 494),
    Label(0x48A, 0x51E6A4, 568, 494),
    Label(0x48B, 0x51E6A4, 233, 522),
    Label(0x48C, 0x51E6A4, 568, 522),
    Label(0x4A3, 0x51E3C4, 98, 415),
    Label(0x4A4, 0x51E3C4, 367, 415),
    Label(0x4A9, 0x51E3C4, 98, 471),
    Label(0x4AA, 0x51E3C4, 367, 471),
    Label(0x4A6, 0x51E3C4, 433, 415),
    Label(0x4A7, 0x51E3C4, 702, 415),
    Label(0x4AC, 0x51E3C4, 433, 471),
    Label(0x4AD, 0x51E3C4, 702, 471),
];

// ---- Rust version ----

/// Returns the carry flag of the last text drawn. Refuses a missing car object and a title
/// that does not fit the text buffer.
pub unsafe fn draw_service_area(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        game_write(CLEARED_RVA, 0u32);

        let mut title = Vec::new();
        push_string(&mut title, TITLE_BEFORE);
        title.push(b' ');
        title.extend(append_u32(game_read::<u32>(CAR_INDEX_RVA).wrapping_add(1)));
        title.push(b' ');
        push_string(&mut title, TITLE_AFTER);
        title.push(0);
        if title.len() > TEXT_LEN {
            return Err(BadInput("the title is longer than the text buffer").into());
        }
        std::ptr::copy_nonoverlapping(title.as_ptr(), text(), title.len());
        draw_text_at(font_at(TITLE_FONT), text(), 0x190, 0xE, canvas)?;

        draw_service_time(canvas)?;

        let Some(car) = current_car() else {
            return Err(BadInput("no current car object: the original reads through null").into());
        };
        let field = |at: u32| (car.wrapping_add(at) as *const i32).read_unaligned();
        // ebx as the original has it: the sprites get it as their palette.
        let mut ebx = 0u32;
        for step in &STEPS {
            match *step {
                Label(id, font, x, y) => {
                    ebx = draw_string_by_id_at(id, font_at(font), x, y, canvas)?.text as u32;
                }
                LabelIf(car_offset, id, font, x, y) => {
                    if field(car_offset) != 0 {
                        ebx = draw_string_by_id_at(id, font_at(font), x, y, canvas)?.text as u32;
                    }
                }
                DamageBar(x, y, car_offset) => {
                    Bar {
                        x,
                        y,
                        percent: field(car_offset),
                        ..DAMAGE_BAR
                    }
                    .write();
                    blit_bar(&mut |call| canvas.blit_glyph(call))?;
                }
                Sprite(glyph, x, y) => {
                    canvas.blit_glyph(BlitCall::new(glyph, x, y, ebx, sprite_table()))?;
                }
            }
        }

        let setting = game_read::<u32>(SETTING_RVA).wrapping_add(SETTING_FIRST_STRING);
        game_write(WORK_RVA, setting);
        draw_string_by_id_at(setting, font_at(SETTING_FONT), 0x238, 0x152, canvas)?;

        // The markers get the car in edi and, as their palette, the ebx the repair times leave.
        let times = draw_repair_times(canvas)?;
        let regs = Regs {
            ebx: times.text as u32,
            edi: car,
            ..Regs::default()
        };
        for marker in [
            draw_brake_balance_marker::MARKER,
            draw_steering_sensitivity_marker::MARKER,
            draw_ride_height_marker::MARKER,
            draw_suspension_stiffness_marker::MARKER,
            draw_ride_height_previous_marker::MARKER,
            draw_suspension_stiffness_previous_marker::MARKER,
        ] {
            draw_marker(marker, &regs, canvas)?;
        }
        Ok(draw_inspection_status(regs.ebx, canvas)?.carry == Some(true))
    }
}

// ---- hook ----

const STATE_RVAS: [usize; 12] = [
    CLEARED_RVA,
    WORK_RVA,
    rva(0x7559AC), // seconds
    rva(0x75ACC0), // the service time penalty
    rva(0x75AC80), // the car condition
    rva(0x755224), // the bar globals
    rva(0x755228),
    rva(0x75522C),
    rva(0x755230),
    rva(0x755234),
    rva(0x755238),
    rva(0x75AC8C), // an input, here to show it is left alone
];

fn state() -> Vec<u8> {
    let mut s = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
    s.extend_from_slice(unsafe { from_raw_parts(line_buffer(), 0x80) });
    push_dwords(&mut s, &STATE_RVAS);
    s
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_service_area(canvas) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test: one case, the service area as it is, with the game's own data ----

/// The fonts the function and what it calls use.
const FONTS: [usize; 10] = [
    0x51E8A4, 0x51E6A4, 0x51E3E4, 0x51E3C4, 0x51E404, 0x51E6E4, 0x51E584, 0x51E5A4, 0x51E524,
    0x51E544,
];

/// Why the current state cannot be drawn by the original without risk, or None.
unsafe fn unusable() -> Option<&'static str> {
    unsafe {
        if blob().is_null() || !FONTS.iter().all(|&f| font_is_loaded(f)) {
            return Some("the string table or the fonts are not loaded");
        }
        if sprite_table() == 0 {
            return Some("the sprite table is not loaded");
        }
        let Some(car) = current_car() else {
            return Some("there is no current car object");
        };
        let setting = game_read::<u32>(SETTING_RVA);
        if setting > 0x40 || !is_table_id(setting + SETTING_FIRST_STRING) {
            return Some("dword_75AC9C does not select a string");
        }
        // A bar whose width * percent is 0 with a non-zero percent is a fatal error, so only
        // values that look like percentages are let through.
        let sane = |p: i32| (-1000..=1000).contains(&p);
        let field = |at: u32| (car.wrapping_add(at) as *const i32).read_unaligned();
        let bars = (0xD4..=0x108).step_by(4).chain([0x110, 0x11C]);
        let edits = [0x75ACA8, 0x75ACAC, 0x75ACB4, 0x75ACB8];
        if !bars.into_iter().all(|at| sane(field(at)))
            || !edits.iter().all(|&at| sane(game_read(rva(at))))
        {
            return Some("the car's values are not percentages");
        }
        None
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable().or_else(|| unsafe { unusable() }) {
        return t.skip(why);
    }
    let saved_text = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
    let saved_line = unsafe { from_raw_parts(line_buffer(), 0x80) }.to_vec();
    let saved: Vec<u32> = STATE_RVAS
        .iter()
        .map(|&at| unsafe { game_read(at) })
        .collect();

    let restore = || unsafe {
        std::ptr::copy_nonoverlapping(saved_text.as_ptr(), text(), TEXT_LEN);
        set_line(&saved_line);
        for (&at, &v) in STATE_RVAS.iter().zip(&saved) {
            game_write(at, v);
        }
    };
    check_draw(
        &mut t,
        "the service area as it is",
        call_original,
        Regs::SENTINEL,
        &restore,
        &rust,
        &state,
    );
    restore();
    t.finish()
}
