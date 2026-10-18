//! `race::mud_texture_turn` (0x44A087): the mud's colour packed into `dword_623D34`
//! (red `dword_60AD00` << 16, its second byte replaced by green's low byte and its low one by
//! blue's: `shl eax, 10h; mov ah, bl; mov al, cl`), `dword_62390C` into `dword_623910` and
//! `dword_623918` into `dword_62391C`, both 0, and `race::update_mud_texture`.
//!
//! In:  nothing
//! Out: the registers as `update_mud_texture` leaves them; no flags read
//!
//! See `race/mud.rs`. One caller, `race::mud_frame`. Its call through `part` (0, in the
//! game the Rust version with its own calls). Refuses what `update_mud_texture` refuses,
//! before anything (its checks first). Shadow mode runs the original alone (the device is
//! told). Self-test: a copy whose call goes to a stand-in, against the Rust version, on
//! colours and values by lot; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::race::mud::{copy, run_both, test_part};
use crate::race::muddy_texel::texture_and_mud;
use crate::race::tint_to_mud_colour::{MUD_COLOUR_RVA, mud_colour};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::mud_texture_turn", rva(0x44A087), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PACKED_RVA: usize = rva(0x623D34); // dword_623D34
const MOVED: [(usize, usize); 2] = [
    (rva(0x62390C), rva(0x623910)), // dword_62390C into dword_623910
    (rva(0x623918), rva(0x62391C)), // dword_623918 into dword_62391C
];

/// A call by place: 0 `race::update_mud_texture`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn mud_texture_turn(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    texture_and_mud()?;
    let mut r = *entry;
    let [red, green, blue] = mud_colour();
    r.eax = (red << 16) & 0xFFFF_0000 | (green & 0xFF) << 8 | (blue & 0xFF);
    r.ebx = green;
    r.ecx = blue;
    unsafe {
        game_write(PACKED_RVA, r.eax);
        for (from, to) in MOVED {
            game_write(to, game_read::<u32>(from));
            game_write(from, 0u32);
        }
    }
    part(0, &mut r)?;
    Ok(r)
}

/// In the game: `update_mud_texture`'s Rust version with its own calls.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::update_mud_texture::{game_part, update_mud_texture};
    *r = unsafe { update_mud_texture(r, &mut game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { mud_texture_turn(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn regions() -> Vec<Region> {
    let mut v = vec![at(PACKED_RVA)];
    for (from, to) in MOVED {
        v.extend([at(from), at(to)]);
    }
    v
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, dword_60AD00
    let copy = match copy(
        0x44A087,
        0x4F,
        &[0xA1, 0x00, 0xAD, 0x60, 0x00],
        &[0x49],
        &[(0x44A12E, 0)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mud = crate::race::muddy_texel::TestMud::new();
    let inputs = [
        (crate::framework::game_addr(MUD_COLOUR_RVA) as u32, 12),
        at(crate::race::muddy_texel::TEXTURE_RVA),
    ];
    let all: Vec<Region> = [&regions()[..], &inputs[..]].concat();
    let saved = capture(&all);
    let regions = regions();
    let mut x = 0xC0FF_EE11u32;
    for case in 0..8u32 {
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x
        };
        let colour = if case == 0 {
            [0x12u32, 0x34, 0x56]
        } else {
            [next(), next(), next()]
        };
        let moved = [next(), next()];
        run_both(
            &mut t,
            &format!("colour {colour:x?}, moved {moved:x?}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(MUD_COLOUR_RVA, colour);
                game_write(crate::race::muddy_texel::TEXTURE_RVA, case % 4);
                game_write(MOVED[0].0, moved[0]);
                game_write(MOVED[1].0, moved[1]);
            },
            &|r| unsafe { mud_texture_turn(r, &mut test_part) },
        );
    }
    restore(&all, &saved);
    drop(mud);
    t.finish()
}
