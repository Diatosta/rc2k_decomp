//! `marks::draw_glow` (0x4D2DED): unless `dword_93CC70` is 1, render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1 (`scene::set_render_state`), `scene::set_texture` with
//! `dword_5D1E34`, and `marks::draw_turned_sprite` at (`flt_93CC7C` + `flt_93CCA8`,
//! `flt_93CC78` + `flt_93CCAC`, `flt_93CC74`) (st(0) first, the sums unrounded), angle
//! `dword_93CCA4`, colour -1, size 0x10000. A glow over the scene, by the look of it (a
//! reading). (Its first lines, a developer's key for the switch, are jumped over.)
//!
//! In:  nothing
//! Out: the registers as the sprite leaves them (eax `dword_5D1E34` going in)
//!
//! Caller: `marks::run_frame` (0x4D1693). Its calls through `part` (1 a render state, 2
//! the texture, 3 the sprite with its three values); in the game their Rust versions. Its
//! first instruction is a short jump: private copy (seven calls relocated), manual
//! patch. Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go
//! to stand-ins (the sprite's taking its values off), against the Rust version on the
//! same, the switch on and off.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::math::x87;
use crate::scene::matrix3::extended;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::draw_glow", rva(0x4D2DED), thunk, &ORIGINAL)
    .private_copy(0xB7)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLS: [usize; 7] = [0x38, 0x47, 0x56, 0x65, 0x74, 0x7E, 0xB1];
const OFF_RVA: usize = rva(0x93CC70); // dword_93CC70: 1 = none
const TEXTURE_RVA: usize = rva(0x5D1E34); // dword_5D1E34
/// flt_93CC74 .. flt_93CC7C, flt_93CCA8, flt_93CCAC, dword_93CCA4.
const PLACE: [usize; 3] = [rva(0x93CC74), rva(0x93CC78), rva(0x93CC7C)];
const OFFSET: [usize; 2] = [rva(0x93CCA8), rva(0x93CCAC)];
const ANGLE_RVA: usize = rva(0x93CCA4);
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// A call by place (see the header), with the x87 values it takes (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_glow(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(OFF_RVA) == 1 {
            return Ok(g);
        }
        for (value, state) in STATES {
            g.eax = value;
            g.ebx = state;
            part(1, &mut g, &[])?;
        }
        g.eax = game_read(TEXTURE_RVA);
        part(2, &mut g, &[])?;
        let place = [
            x87::add_80(game_read(PLACE[2]), game_read(OFFSET[0])),
            x87::add_80(game_read(PLACE[1]), game_read(OFFSET[1])),
            extended(game_read(PLACE[0])),
        ];
        g.ebx = game_read(ANGLE_RVA);
        g.ecx = 0xFFFF_FFFF;
        g.edx = 0x1_0000;
        part(3, &mut g, &place)?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => {
                let (value, state) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(r, value, state);
            }
            2 => crate::scene::set_texture::set_texture_in_game(r),
            _ => {
                use crate::marks::draw_turned_sprite as sprite;
                let left =
                    sprite::draw_turned_sprite(r, [v[0], v[1], v[2]], &mut sprite::game_part)?;
                *r = Regs { esp: r.esp, ..left };
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_glow(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_glow(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

/// A call as the test notes it: place, eax .. edx, the x87 values.
type Noted = (u32, [u32; 4], Vec<[u8; 10]>);

static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

fn note(place: u32, r: &Regs, v: &[[u8; 10]]) {
    NOTED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx], v.to_vec()));
}

fn note_regs(place: u32, r: &mut Regs) {
    note(place, r, &[]);
}

extern "C" fn note_three(r: *const Regs, v: *const [u8; 10]) {
    note(3, unsafe { &*r }, unsafe {
        std::slice::from_raw_parts(v, 3)
    });
}

/// The test's stand-in for the sprite: its three values taken off and noted.
#[unsafe(naked)]
unsafe extern "C" fn sprite_stand_in() {
    core::arch::naked_asm!(
        "lea esp, [esp - 30]",
        "fstp tbyte ptr [esp]",
        "fstp tbyte ptr [esp + 10]",
        "fstp tbyte ptr [esp + 20]",
        "pushfd",
        "pushad",
        "mov eax, esp",
        "lea ecx, [esp + 36]",
        "push ecx",
        "push eax",
        "call {note}",
        "add esp, 8",
        "popad",
        "popfd",
        "lea esp, [esp + 30]",
        "ret",
        note = sym note_three,
    )
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    set_handler(note_regs);
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x4D2DED)),
            0xB7,
            // jmp short +0x22, test byte_4F6B86, 0FFh
            &[0xEB, 0x22, 0xF6, 0x05, 0x86, 0x6B, 0x4F, 0x00, 0xFF],
            &CALLS,
            &|c| match c + 0x40_0000 {
                0x44EB72 => Some(stand_in(1)),
                0x44E77E => Some(stand_in(2)),
                0x4CBD12 => Some(sprite_stand_in as *const () as usize),
                _ => None,
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut vars = vec![at(OFF_RVA), at(TEXTURE_RVA), at(ANGLE_RVA)];
    vars.extend(PLACE.map(at));
    vars.extend(OFFSET.map(at));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..3u32 {
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(OFF_RVA, u32::from(case == 2));
            game_write(TEXTURE_RVA, 0x30 + case);
            game_write(ANGLE_RVA, case * 0x155);
            for (n, r) in PLACE.into_iter().enumerate() {
                game_write(r, (n as f32 * 3.5 - 1.0 + case as f32).to_bits());
            }
            game_write(OFFSET[0], (1.0f32 / 3.0).to_bits());
            game_write(OFFSET[1], (1e-3f32 * case as f32).to_bits());
        };
        prepare();
        NOTED.lock().unwrap().clear();
        let mut ro = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), std::mem::take(&mut *NOTED.lock().unwrap()));
        prepare();
        let mine = unsafe {
            draw_glow(&Regs::SENTINEL, &mut |p, r, v| {
                note(p, r, v);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            std::mem::take(&mut *NOTED.lock().unwrap()),
        );
        t.check(theirs == mine, || {
            format!("case {case}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&vars, &saved);
    t.finish()
}
