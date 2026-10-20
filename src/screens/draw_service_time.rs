//! `screens::draw_service_time` (0x4A5DE7): draws the two time readouts at the top of the
//! service area, from the time budget and the time of the repairs chosen.
//!
//! In:  nothing
//! Out: every register preserved; carry as the last `draw_text` leaves it
//! While `budget - repairs` is at least 900: "<string 0x48D>:  <budget - 900>" on the left
//! and "<string 0x48E>:  <budget - 900 - repairs>" on the right. Below that:
//! "<string 0x490>:  <overrun>" and "<string 0x491>:  <penalty>", where the overrun is
//! `900 - (budget - repairs)` and the penalty is 10 for every started 60 of overrun.
//! All four values are printed with `append_clock`. It leaves the penalty in `dword_75ACC0`.
//!
//! Screen: seen in a screenshot of the championship's service area (2026-10-10): "TEMPO DISPONÍVEL: 40:00" at
//! the top left and "TEMPO RESTANTE: 40:00" at the top right (time available, time left).
//! The overrun pair of labels has not been seen.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable, push_dwords};
use crate::screens::{font_is_loaded, line_buffer, set_line};
use crate::service::state::{REPAIR_TOTAL_RVA, TIME_BUDGET_RVA};
use crate::strings::string_table::{blob, is_table_id};
use crate::text::draw_text::Canvas;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_service_time",
    rva(0x4A5DE7),
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

const PENALTY_RVA: usize = rva(0x75ACC0); // dword_75ACC0
const WORK_A_RVA: usize = rva(0x7559AC); // seconds: a work variable
const WORK_B_RVA: usize = rva(0x7559EC); // dword_7559EC: a work variable

// (label string, font) for the four readouts.
const AVAILABLE: (u32, usize) = (0x48D, 0x51E584);
const REMAINING: (u32, usize) = (0x48E, 0x51E5A4);
const OVERRUN: (u32, usize) = (0x490, 0x51E524);
const PENALTY: (u32, usize) = (0x491, 0x51E544);

// ---- Rust version ----

/// Returns the carry flag of the last `draw_text`.
pub unsafe fn draw_service_time(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::service::service_time`.
    unsafe {
        rc2k_core::screens::service::service_time(
            &mut crate::screens::game_screen::ServiceScreen::new(canvas),
            game_read(TIME_BUDGET_RVA),
            game_read(REPAIR_TOTAL_RVA),
        )
    }
}

// ---- hook ----

fn state() -> Vec<u8> {
    let mut s = unsafe { from_raw_parts(line_buffer(), 0x80) }.to_vec();
    push_dwords(&mut s, &[PENALTY_RVA, WORK_A_RVA, WORK_B_RVA]);
    s
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_service_time(canvas) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test: budgets and repair times on both sides of the limit ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let readouts = [AVAILABLE, REMAINING, OVERRUN, PENALTY];
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if blob().is_null()
        || !readouts
            .iter()
            .all(|&(id, font)| is_table_id(id) && font_is_loaded(font))
    {
        return t.skip("the string table or the fonts are not loaded");
    }
    let saved = (state(), unsafe {
        (
            game_read::<i32>(TIME_BUDGET_RVA),
            game_read::<i32>(REPAIR_TOTAL_RVA),
        )
    });
    let saved_line = unsafe { from_raw_parts(line_buffer(), 0x80) }.to_vec();

    for (budget, repairs) in [
        (3600, 0),
        (3600, 2700), // exactly at the limit
        (3600, 2701), // one over
        (3600, 2760),
        (3600, 2761),
        (3600, 3600),
        (3600, 9000),
        (900, 0),
        (1200, 300),
        (0, 0),
    ] {
        let prepare = || unsafe {
            game_write(TIME_BUDGET_RVA, budget);
            game_write(REPAIR_TOTAL_RVA, repairs);
            game_write(PENALTY_RVA, 0x5EED_0001u32);
            game_write(WORK_A_RVA, 0x5EED_0002u32);
            game_write(WORK_B_RVA, 0x5EED_0003u32);
            set_line(&saved_line);
        };
        check_draw(
            &mut t,
            &format!("budget={budget} repairs={repairs}"),
            call_original,
            Regs::SENTINEL,
            &prepare,
            &rust,
            &state,
        );
    }

    unsafe {
        game_write(TIME_BUDGET_RVA, saved.1.0);
        game_write(REPAIR_TOTAL_RVA, saved.1.1);
        set_line(&saved_line);
        let at = saved.0.len() - 12;
        for (i, rva) in [PENALTY_RVA, WORK_A_RVA, WORK_B_RVA].iter().enumerate() {
            let bytes: [u8; 4] = saved.0[at + i * 4..at + i * 4 + 4]
                .try_into()
                .unwrap_or([0; 4]);
            game_write(*rva, u32::from_le_bytes(bytes));
        }
    }
    t.finish()
}
