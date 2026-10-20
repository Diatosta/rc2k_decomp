//! `service::load_background` (0x470F55): the service area's picture for the current
//! player's car: the player (`g_players`[`dword_75598C`]), its car (+8; car 0x21 takes car
//! 0x17's picture), the picture `off_75A52E`[car] (`files\g\graphics\screens\servNN.png`, 60
//! of them) loaded (`gfx::load_picture`) and converted (`gfx::convert_picture`); a failed
//! conversion (carry) is fatal error 0x80712435.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the service area's screens (0x47296A, 0x472B53, 0x4A1C0F). Its calls through
//! `part` (0 the loading with eax the path, 1 the conversion; in the game their Rust
//! versions). Refuses (before anything) a current player past the four or missing, and a car
//! past the table. Shadow mode runs the original alone. Self-test: a copy whose two calls go
//! to stand-ins, against the Rust version: cars 0, 0x17, 0x21 and 0x3B, the conversion
//! failing.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::parts::{Answer, Part, answers, copy, dispatch_parted, run_both, test_part};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::load_background", rva(0x470F55), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT_PLAYER_RVA: usize = rva(0x75598C); // dword_75598C
const PICTURES_RVA: usize = rva(0x75A52E); // off_75A52E: the service pictures' paths
const PICTURES: u32 = 60;
const CALLEES: [usize; 2] = [0x411863, 0x406126]; // gfx::load_picture, gfx::convert_picture
const NOT_CONVERTED: FatalError = FatalError {
    code: 0x8071_2435,
    message: "",
};

/// The Rust version: every register kept.
pub unsafe fn load_background(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let player = crate::menu::player::player_at(unsafe { game_read(CURRENT_PLAYER_RVA) });
    if player == 0 {
        return Err(BadInput("no current player (or one past the four)").into());
    }
    let mut car = unsafe { ((player + 8) as *const u32).read_unaligned() };
    if car == 0x21 {
        car = 0x17;
    }
    if car >= PICTURES {
        return Err(BadInput("a car past the service pictures' table").into());
    }
    let mut r = Regs {
        eax: unsafe { game_read(PICTURES_RVA + 4 * car as usize) },
        ebx: car,
        edi: player,
        ..*entry
    };
    part(0, &mut r)?;
    part(1, &mut r)?;
    if r.carry() {
        return Err(NOT_CONVERTED.into());
    }
    Ok(*entry)
}

fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => {
                crate::gfx::load_picture::load_picture(r.eax as *mut u8)?;
            }
            _ => {
                crate::gfx::convert_picture::convert_picture_now()?;
                r.set_carry(false);
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parted(
        &HOOK,
        r,
        call_original,
        &|r, part| unsafe { load_background(r, part) },
        game_part,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let player = crate::menu::player::player_at(0);
    if player == 0 {
        return t.skip("no first player");
    }
    let copy = match copy(
        0x470F55,
        0x35,
        &[0x60, 0x50, 0xA1, 0x8C, 0x59, 0x75, 0x00],
        &[0x23, 0x28],
        &CALLEES,
        &[0x2F],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let vars = [at(CURRENT_PLAYER_RVA), (player + 8, 4)];
    let saved = capture(&vars);
    for (car, fails) in [
        (0u32, false),
        (0x17, false),
        (0x21, false),
        (0x3B, false),
        (3, true),
    ] {
        answers(&[(
            1,
            &[Answer {
                carry: Some(fails),
                ..Answer::default()
            }],
        )]);
        run_both(
            &mut t,
            &format!("car {car:#x}, the conversion failing {fails}"),
            &copy,
            &vars,
            &|| unsafe {
                game_write(CURRENT_PLAYER_RVA, 0u32);
                ((player + 8) as *mut u32).write_unaligned(car);
            },
            Regs::SENTINEL,
            false,
            &|r| unsafe { load_background(r, &mut test_part) },
        );
    }
    answers(&[]);
    restore(&vars, &saved);
    t.finish()
}
