//! `menu::fit_player_names` (0x411445): cuts each player's name until it is no wider than
//! 350 pixels.
//!
//! In:  ecx (the x the first measuring is done with)
//! Out: every register preserved
//! For each of the four names in `g_playerNames` (`menu::fit_name`, 0x411474, a plain
//! function here): the name is measured with the font at 0x51E824 (`text::measure_text`)
//! and, while it is wider than 0x15E, loses its last character and is measured again.
//! The x given to a measuring is the width the one before gave (the first gets the
//! caller's ecx): the original leaves it in ecx, kept.
//! One caller, the main menu's set-up `menu::set_up_front_end`.
//! Refuses a name that would be cut to before its start, and what the measuring refuses.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_ptr, hook_asm, rva};
use crate::gfx::blit_glyph::record_calls;
use crate::menu::player_names::{NAME_LEN, PLAYERS, name};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore};
use crate::text::draw_text::GameCanvas;
use crate::text::font::Font;
use crate::text::glyph_kerning::{STATE_LEN, state_ptr};
use crate::text::measure_string_by_id::{MEASURING_FONT, measuring_unavailable};
use crate::text::measure_text::measure_text_at;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::fit_player_names", rva(0x411445), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const WIDEST: u32 = 0x15E;
/// More characters than a name is looked through for its end.
const LONGEST: usize = 0x1000;

// ---- Rust version ----

/// `menu::fit_name` (0x411474). Returns the width of the name as it is left.
pub unsafe fn fit_name(text: *mut u8, x: u32) -> Result<u32, Refused> {
    unsafe {
        let Some(mut len) = (0..LONGEST).find(|&n| text.add(n).read() == 0) else {
            return Err(BadInput("a name without its end").into());
        };
        let font: *mut Font = game_ptr(rva(MEASURING_FONT));
        let mut x = x;
        loop {
            text.add(len).write(0);
            x = measure_text_at(font, text, x as i32, &mut GameCanvas)?.width;
            if x <= WIDEST {
                return Ok(x);
            }
            if len == 0 {
                return Err(BadInput("an empty name that measures too wide").into());
            }
            len -= 1;
        }
    }
}

pub unsafe fn fit_player_names(x: u32) -> Result<(), Refused> {
    let mut x = x;
    for player in 0..PLAYERS {
        x = unsafe { fit_name(name(player), x) }?;
    }
    Ok(())
}

// ---- hook ----

fn regions() -> [Region; 3] {
    [
        (name(0) as u32, PLAYERS * NAME_LEN),
        (game_ptr::<u8>(rva(MEASURING_FONT)) as u32, 0x20),
        (state_ptr() as u32, STATE_LEN),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { fit_player_names(r.ecx) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: names of the test's in the game's place, measured with the game's
// font; everything put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = measuring_unavailable() {
        return t.skip(why);
    }
    let regions = regions();
    let saved = capture(&regions);
    let font_before = saved[PLAYERS * NAME_LEN..][..0x20].to_vec();
    let cases: [(&str, [&[u8]; PLAYERS], u32); 4] = [
        ("short names", [b"anna", b"bob", b"", b"x"], 0),
        (
            "wide names",
            [
                b"WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW",
                b"mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm",
                b"iiiiiiiiiiiiiiiiiiiiiiiiiiiiiii",
                b"W W W W W W W W W W W W W W W W",
            ],
            0,
        ),
        (
            "mixed, measured from another x",
            [
                b"MWMWMWMWMWMWMWMWMWMWMWMWMWMWMWM",
                b"a name of thirty-one characters",
                b"",
                b"QQQQQQQQQQQQQQQQQQQQQ",
            ],
            123,
        ),
        ("one character each", [b"W", b"i", b" ", b"."], 0x7FFF),
    ];
    for (what, names, x) in cases {
        let before = Regs {
            ecx: x,
            ..Regs::SENTINEL
        };
        let run = |rust_side: bool| unsafe {
            restore(&regions, &saved);
            for (player, text) in names.into_iter().enumerate() {
                name(player).write_bytes(0, NAME_LEN);
                std::ptr::copy(text.as_ptr(), name(player), text.len());
            }
            restore(&regions[1..2], &font_before);
            let mut ended = 0u32;
            // Whatever the measuring draws is swallowed.
            record_calls(true, || {
                if rust_side {
                    ended = rust(&before).is_err() as u32;
                } else {
                    let mut ro = before;
                    call_original(&mut ro);
                    ended = !ro.preserved_except(&before, 0) as u32;
                }
            });
            (ended, capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 == 0, || {
            let names_of = |left: &Vec<u8>| crate::strings::utils::bytes_str(&left[..PLAYERS * NAME_LEN]);
            format!(
                "{what}: original ended {} names {}; rust ended {} names {}; first different byte left {:?}",
                theirs.0,
                names_of(&theirs.1),
                mine.0,
                names_of(&mine.1),
                theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
