//! `screens::draw_repair_times` (0x4A70D7): draws sixteen values of the service area as
//! clocks, in two columns of eight: `dword_75AB60[0 .. 16]`, left column first in each row.
//!
//! In:  nothing
//! Out: what the last `draw_text` leaves: carry, eax = 0, ebx = after the text's NUL,
//!      ecx = 441, edx = 452; esi, edi and ebp preserved
//! Rows are at y = 110 .. 284 in steps of 29, and a last one at 452; columns at x = 106 and
//! 441.
//!
//! Screen: seen in a screenshot of the championship's service area (2026-10-10): the time under each of the
//! fourteen parts of the repair section (for example 10:19 under the gearbox, 1:35 under the
//! tyres), and in the last row the 1:20 under "ride height" and "suspension stiffness".

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable};
use crate::screens::{font_at, font_is_loaded, line_buffer, set_line};
use crate::strings::append_clock::append_clock;
use crate::text::draw_text::{Canvas, Outcome, draw_text_at};
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_repair_times",
    rva(0x4A70D7),
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

const TIMES_RVA: usize = rva(0x75AB60); // dword_75AB60[16]
const COUNT: usize = 16;
const FONT: usize = 0x51E3E4;

const COLUMNS: [i32; 2] = [0x6A, 0x1B9];
const ROWS: [i32; 8] = [0x6E, 0x8B, 0xA8, 0xC5, 0xE2, 0xFF, 0x11C, 0x1C4];

fn time_rva(i: usize) -> usize {
    TIMES_RVA + i * 4
}

// ---- Rust version ----

/// Returns what the last `draw_text` returned.
pub unsafe fn draw_repair_times(canvas: &mut dyn Canvas) -> Result<Outcome, Refused> {
    let mut last = None;
    for i in 0..COUNT {
        unsafe {
            let mut line = append_clock(game_read(time_rva(i)));
            line.push(0);
            set_line(&line);
            let (x, y) = (COLUMNS[i % 2], ROWS[i / 2]);
            last = Some(draw_text_at(font_at(FONT), line_buffer(), x, y, canvas)?);
        }
    }
    // COUNT is not 0, so there is always a last one.
    last.ok_or(Refused::BadInput(crate::errors::BadInput(
        "nothing to draw",
    )))
}

// ---- hook ----

fn state() -> Vec<u8> {
    unsafe { from_raw_parts(line_buffer(), 0x80) }.to_vec()
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::of_text(&unsafe { draw_repair_times(canvas) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test: three sets of sixteen values ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if !font_is_loaded(FONT) {
        return t.skip("the font is not loaded");
    }
    let saved: Vec<u32> = (0..COUNT)
        .map(|i| unsafe { game_read(time_rva(i)) })
        .collect();
    let saved_line = state();

    let sets: [[u32; COUNT]; 3] = [
        [0; COUNT],
        std::array::from_fn(|i| (i as u32 * 37 + 5) % 600),
        std::array::from_fn(|i| [59, 60, 61, 3599, 3600, 36_000, 123_456, 1][i % 8]),
    ];
    for (n, set) in sets.iter().enumerate() {
        let prepare = || unsafe {
            for (i, v) in set.iter().enumerate() {
                game_write(time_rva(i), *v);
            }
            set_line(&saved_line);
        };
        check_draw(
            &mut t,
            &format!("set {n}"),
            call_original,
            Regs::SENTINEL,
            &prepare,
            &rust,
            &state,
        );
    }

    unsafe {
        for (i, v) in saved.iter().enumerate() {
            game_write(time_rva(i), *v);
        }
        set_line(&saved_line);
    }
    t.finish()
}
