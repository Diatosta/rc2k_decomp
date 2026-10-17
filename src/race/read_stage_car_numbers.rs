//! `race::read_stage_car_numbers` (0x437971): the stage's line of the cars' data text
//! ('l' or 'L' and the stage's two digits) and its three-digit numbers, four apart, into
//! `g_stageCarNumbers` (0x601430) and their sum into `g_stageCarTotal` (0x601C4C).
//!
//! In:  nothing
//! Out: ecx = the place reached, ebp = the count, eax = the last number parsed (0 for a
//!      failed one; the entry's when no line was found); the rest preserved
//!
//! See `race/race_cars.rs`. One caller, `sub_43603C` (modes 0 and 2). Refuses more than
//! 0x100 numbers (worked out first). Shadow mode: the Rust version, the variables put
//! back, the original's, compared. Self-test: texts of the test's in the game's buffer
//! (stage found and not, lower case, a stage of 0 against a line without digits, numbers
//! to the text's end), all put back; the refusal on the Rust version alone.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::race_cars::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::read_stage_car_numbers",
    rva(0x437971),
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { read_stage_car_numbers(r) }?;
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &number_regions(), &rust);
}

fn text_of(lines: &[u8]) -> Vec<u8> {
    let mut bytes = vec![b' '; TEXT_LEN];
    bytes[..lines.len()].copy_from_slice(lines);
    bytes
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let text = (game_addr(TEXT_RVA) as u32, TEXT_LEN);
    let regions = number_regions();
    let mut kept = regions.clone();
    kept.extend([text, crate::service::harness::at(STAGE_RVA)]);
    let saved = capture(&kept);
    let mut end = text_of(b"L21 100 200");
    let n = TEXT_LEN;
    end[n - 13..].copy_from_slice(b"L33 123 456 7");
    let cases: Vec<(&str, u32, Vec<u8>)> = vec![
        (
            "found",
            21,
            text_of(b"rally\r\nL20 111 222\r\nL21 123 045 999x\r\nL22 1"),
        ),
        (
            "lower case",
            22,
            text_of(b"l21 123\r\nl22 300 301 302 303\r\n"),
        ),
        ("not there", 40, text_of(b"L21 123 456\r\nL22 789\r\n")),
        ("stage 0, no digits", 0, text_of(b"Lxx 555 666 aaa")),
        ("no L", 21, text_of(b"21 123 456")),
        ("at the end", 33, end.clone()),
        ("L last", 33, {
            let mut b = text_of(b"");
            b[n - 1] = b'L';
            b
        }),
    ];
    for (what, stage, bytes) in &cases {
        check_state(
            &mut t,
            &format!("{what}, stage {stage}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&[text], bytes);
                game_write(STAGE_RVA, *stage);
                restore(&regions, &vec![0x5A; 8 + 4 * MOST_NUMBERS]);
            },
            &regions,
            &rust,
        );
    }
    // More numbers than the table holds: the Rust version alone.
    let mut many = b"L21 ".to_vec();
    for _ in 0..0x120 {
        many.extend_from_slice(b"123 ");
    }
    restore(&[text], &text_of(&many));
    unsafe { game_write(STAGE_RVA, 21u32) };
    let refused = unsafe { read_stage_car_numbers(&Regs::SENTINEL) }.is_err();
    t.check(refused, || "0x120 numbers: not refused".to_string());
    restore(&kept, &saved);
    t.finish()
}
