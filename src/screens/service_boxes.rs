//! The service area's boxes (`screens::draw_service_boxes` and its four frames): flat
//! see-through quads (`scene::draw_tinted_quad`) laid out by long tables of the original's
//! code. The hooks' bodies are written out from the disassembly by a translator (each quad
//! the original's six stores and its call; each `cmp` / `jz` / `jnz` / `jmp` a test and a
//! `break` out of a labelled block, every jump forward); this file holds what they share.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;

/// A call by place: 1 `draw_tinted_quad`, 2 .. the function's other calls.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A variable by its address in IDA.
pub fn rd(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

pub fn set(a: usize, v: u32) {
    unsafe { game_write(rva(a), v) };
}

pub fn addr(a: usize) -> u32 {
    game_addr(rva(a)) as u32
}

/// The dword at `p` (the original's `[esi]`).
pub fn at_esi(p: u32) -> u32 {
    unsafe { (p as *const u32).read_unaligned() }
}

const QUAD_IN: usize = 0x623DE8; // dword_623DE8: colour, see-through; x at +0x28 / +0x2C, y at +0x38 / +0x3C

/// One quad: x .. x + w, y .. y + h, the colour and see-through, then `draw_tinted_quad`
/// (eax the see-through going in, as the original's last `mov`).
#[allow(clippy::too_many_arguments)] // the original's six stores and its call, one to one
pub fn quad(
    g: &mut Regs,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    colour: u32,
    see: u32,
    part: &mut Part,
) -> Result<(), Refused> {
    set(QUAD_IN + 0x28, x);
    set(QUAD_IN + 0x38, y);
    set(QUAD_IN + 0x2C, x.wrapping_add(w));
    set(QUAD_IN + 0x3C, y.wrapping_add(h));
    set(QUAD_IN, colour);
    set(QUAD_IN + 4, see);
    g.eax = see;
    part(1, g)
}

/// In the game: `draw_tinted_quad`'s Rust version for place 1.
pub fn quad_part(r: &mut Regs) -> Result<(), Refused> {
    use crate::scene::{draw_flat_quad, draw_tinted_quad};
    *r = unsafe { draw_tinted_quad::draw_tinted_quad(r, &mut draw_flat_quad::game_part) }?;
    Ok(())
}

/// A call as the tests note it: the place, eax / ebx / ecx / esi, the quad's inputs.
type Noted = (u32, [u32; 4], [u32; 0x10]);

static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

pub fn note(place: u32, r: &mut Regs) {
    let q: [u32; 0x10] = unsafe { game_read(rva(QUAD_IN)) };
    NOTED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.esi], q));
}

fn take() -> Vec<Noted> {
    std::mem::take(&mut *NOTED.lock().unwrap())
}

/// The variables the tables test (their addresses in IDA and the values they are compared
/// with), and the two flag tables some read by index (0x15 entries each).
pub struct Inputs {
    pub tested: &'static [(usize, &'static [u32])],
    pub read: &'static [usize],
}

const FLAG_TABLES: [usize; 2] = [0x759B6E, 0x759BC2];

/// The test of a box function: a copy whose calls go to stand-ins noting the quads'
/// inputs, against the Rust version, the tested variables by lot over their compared
/// values (and one more), the flag tables by lot.
#[allow(clippy::too_many_arguments)]
pub fn test_boxes(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    places: &'static [(usize, u32)],
    inputs: &Inputs,
    cases: u32,
    rust: &dyn Fn(&Regs, Part) -> Result<Regs, Refused>,
) -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            places
                .iter()
                .find(|&&(k, _)| rva(k) == c)
                .map(|&(_, p)| stand_in(p))
        })
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(note);
    let regions: Vec<Region> = vec![(addr(QUAD_IN), 0x40)];
    let mut vars = regions.clone();
    for &(a, _) in inputs.tested {
        vars.push(at(rva(a)));
    }
    for &a in inputs.read {
        vars.push(at(rva(a)));
    }
    for a in FLAG_TABLES {
        vars.push((addr(a), 0x54));
    }
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let mut seed = 0x2545_F491u32;
    for case in 0..cases {
        let mut lot = || {
            seed = seed.wrapping_mul(0x0101_9E35).wrapping_add(0x3C6E_F372);
            seed >> 8
        };
        let mut pick: Vec<(usize, u32)> = Vec::new();
        for &(a, values) in inputs.tested {
            let v = values[case as usize % values.len()];
            pick.push((a, if lot() % 5 == 0 { v.wrapping_add(1) } else { v }));
        }
        for &a in inputs.read {
            pick.push((a, lot() % 6));
        }
        let flags: Vec<u32> = (0..2 * 0x15).map(|_| lot() & 1).collect();
        let prepare = || {
            restore(&vars, &saved);
            for &(a, v) in &pick {
                set(a, v);
            }
            for (n, a) in FLAG_TABLES.into_iter().enumerate() {
                for k in 0..0x15 {
                    set(a + 4 * k, flags[n * 0x15 + k]);
                }
            }
        };
        prepare();
        take();
        let mut ro = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take(), capture(&regions));
        prepare();
        let mine = rust(&Regs::SENTINEL, &mut |p, r| {
            note(p, r);
            Ok(())
        });
        let mine = (mine.as_ref().ok().map(regs), take(), capture(&regions));
        t.check(theirs == mine, || {
            let first = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!(
                "case {case}: original {:x?} {} calls; rust {:x?} {} calls; first differing call {first:?}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len()
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
