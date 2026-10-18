//! For self-tests of loops on keys, clocks and answers: `install::stand_ins` handlers whose
//! answers follow a script, place by place (each call takes the next answer, then the
//! place's default), noting every call with its registers; the Rust version's `part` calls
//! the same `handler`, so both sides see the same answers.

use crate::framework::{Regs, SelfTest, call_game, game_addr, rva};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture};
use std::collections::VecDeque;
use std::sync::Mutex;

/// What a call does: eax, or only al (the rest kept), the carry and zero flags, and a
/// dword written (address in IDA, value).
#[derive(Clone, Copy, Default)]
pub struct Answer {
    pub eax: Option<u32>,
    pub al: Option<u8>,
    pub carry: bool,
    pub zero: Option<bool>,
    pub write: Option<(usize, u32)>,
}

impl Answer {
    /// A key in al, zero when it is 0 (`input::take_key`).
    pub const fn key(k: u8) -> Self {
        Answer {
            eax: None,
            al: Some(k),
            carry: false,
            zero: Some(k == 0),
            write: None,
        }
    }
    /// eax (a clock).
    pub const fn eax(v: u32) -> Self {
        Answer {
            eax: Some(v),
            al: None,
            carry: false,
            zero: None,
            write: None,
        }
    }
    /// The zero flag alone.
    pub const fn zero(z: bool) -> Self {
        Answer {
            eax: None,
            al: None,
            carry: false,
            zero: Some(z),
            write: None,
        }
    }
    /// A dword written.
    pub const fn write(at: usize, v: u32) -> Self {
        Answer {
            eax: None,
            al: None,
            carry: false,
            zero: None,
            write: Some((at, v)),
        }
    }
}

/// A place's answers in order, and its default after them.
pub type Line = (u32, Vec<Answer>, Answer);

struct State {
    lines: Vec<(u32, VecDeque<Answer>, Answer)>,
    notes: Vec<[u32; 8]>,
}

static STATE: Mutex<State> = Mutex::new(State {
    lines: Vec::new(),
    notes: Vec::new(),
});

/// Starts a run with a script.
pub fn begin(script: &[Line]) {
    let mut s = STATE.lock().unwrap();
    s.lines = script
        .iter()
        .map(|(p, a, d)| (*p, a.iter().copied().collect(), *d))
        .collect();
    s.notes.clear();
}

/// The calls noted since `begin`.
pub fn notes() -> Vec<[u32; 8]> {
    STATE.lock().unwrap().notes.clone()
}

/// The stand-ins' handler (and the Rust side's part): notes the call, gives its answer.
pub fn handler(place: u32, r: &mut Regs) {
    let answer = {
        let mut s = STATE.lock().unwrap();
        s.notes
            .push([place, r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]);
        if s.notes.len() > 10_000 {
            // a loop the script does not end: the clock far on and a key down, so that it
            // ends and the test fails instead of hanging
            Answer {
                eax: Some(0x7FFF_FFFF),
                zero: Some(false),
                ..Answer::default()
            }
        } else {
            s.lines
                .iter_mut()
                .find(|(p, _, _)| *p == place)
                .map(|(_, q, d)| q.pop_front().unwrap_or(*d))
                .unwrap_or_default()
        }
    };
    if let Some(v) = answer.eax {
        r.eax = v;
    }
    if let Some(v) = answer.al {
        r.eax = (r.eax & !0xFF) | u32::from(v);
    }
    r.set_carry(answer.carry);
    if let Some(z) = answer.zero {
        r.set_zero(z);
    }
    if let Some((at, v)) = answer.write {
        unsafe { (game_addr(rva(at)) as *mut u32).write_unaligned(v) };
    }
}

/// The test of a loop: a copy of `address` whose calls to `places`' functions go to the
/// stand-ins, against `rust` on the same script, for each script: the calls, eax .. ebp,
/// carry, zero and `regions` compared; `prepare` before each side.
#[allow(clippy::too_many_arguments)]
pub fn test_loop(
    mut t: SelfTest,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    places: &[(usize, u32)],
    regions: &[Region],
    prepare: &dyn Fn(),
    scripts: &[Vec<Line>],
    before: Regs,
    rust: &dyn Fn(&Regs) -> Regs,
) -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    let copy = match unsafe {
        RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            places
                .iter()
                .find(|&&(k, _)| rva(k) == c)
                .map(|&(_, p)| stand_in(p))
        })
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(handler);
    let regs = |r: &Regs| {
        [
            r.eax,
            r.ebx,
            r.ecx,
            r.edx,
            r.esi,
            r.edi,
            r.ebp,
            u32::from(r.carry()),
            u32::from(r.eflags & 0x40 != 0),
        ]
    };
    for (n, script) in scripts.iter().enumerate() {
        prepare();
        begin(script);
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (notes(), regs(&ro), capture(regions));
        prepare();
        begin(script);
        let left = rust(&before);
        let mine = (notes(), regs(&left), capture(regions));
        t.check(theirs == mine, || {
            format!(
                "script {n}: calls {} / {}, first differing {:?}: {:x?} / {:x?}; registers {:x?} / {:x?}; state same {}",
                theirs.0.len(),
                mine.0.len(),
                theirs.0.iter().zip(&mine.0).position(|(a, b)| a != b),
                theirs.0.iter().zip(&mine.0).find(|(a, b)| a != b).map(|p| p.0),
                theirs.0.iter().zip(&mine.0).find(|(a, b)| a != b).map(|p| p.1),
                theirs.1,
                mine.1,
                theirs.2 == mine.2
            )
        });
    }
    t.finish()
}
