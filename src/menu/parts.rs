//! For the menu set-ups and item actions ported on 2026-10-19 (agent E): the calls a Rust
//! version makes go through a `Part` by place (its callee's place in its list), so that a
//! self-test can run a copy of the original whose calls go to stand-ins of those places and
//! the Rust version on the same stand-ins' notes. Each call is noted with the seven
//! registers; what a stand-in answers (registers, the carry, a game dword written) is set
//! per place, in turn (the last answer stays).

use crate::errors::{FatalError, Refused};
use crate::framework::{Regs, SelfTest, call_game, game_addr, rva};
use crate::install::stand_ins::{WENT_FATAL, fatal_landing, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture};
use std::sync::Mutex;
use std::sync::atomic::Ordering;

/// A call by place.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A Rust version with its calls through a `Part`: the registers it leaves.
pub type Parted<'a> = &'a dyn Fn(&Regs, Part) -> Result<Regs, Refused>;

/// The `dispatch` of such a hook whose calls have effects outside memory (a sound, a
/// screen, a log line): shadow mode runs the original alone; else the Rust version with
/// `game_part`, a refusal to `fatal::raise::refused`.
pub fn dispatch_parted(
    hook: &crate::framework::RawHook,
    r: &mut Regs,
    call_original: crate::screens::harness::CallOriginal,
    rust: Parted,
    game_part: fn(u32, &mut Regs) -> Result<(), Refused>,
) {
    hook.count_call();
    if crate::framework::shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut part = game_part;
    match rust(r, &mut part) {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

/// What a stand-in does after noting the call.
#[derive(Clone, Copy, Debug, Default)]
pub struct Answer {
    pub eax: Option<u32>,
    pub ebx: Option<u32>,
    pub ecx: Option<u32>,
    pub edx: Option<u32>,
    pub esi: Option<u32>,
    pub edi: Option<u32>,
    pub carry: Option<bool>,
    /// A dword written at this address (a screen's answer left in a game variable).
    pub write: Option<(u32, u32)>,
    /// And a second one.
    pub write2: Option<(u32, u32)>,
}

impl Answer {
    pub fn eax(value: u32) -> Self {
        Self {
            eax: Some(value),
            ..Self::default()
        }
    }
    pub fn edi(value: u32) -> Self {
        Self {
            edi: Some(value),
            ..Self::default()
        }
    }
    pub fn carry(on: bool) -> Self {
        Self {
            carry: Some(on),
            ..Self::default()
        }
    }
    pub fn write(at: u32, value: u32) -> Self {
        Self {
            write: Some((at, value)),
            ..Self::default()
        }
    }
    /// Writes to two game variables (RVAs).
    pub fn writes(a: usize, va: u32, b: usize, vb: u32) -> Self {
        Self {
            write: Some((game_addr(a) as u32, va)),
            write2: Some((game_addr(b) as u32, vb)),
            ..Self::default()
        }
    }
    /// Writes to a game variable (an RVA).
    pub fn sets(a: usize, value: u32) -> Self {
        Self::write(game_addr(a) as u32, value)
    }
}

/// (place, eax, ebx, ecx, edx, esi, edi, ebp)
pub type Note = (u32, [u32; 7]);

static CALLED: Mutex<Vec<Note>> = Mutex::new(Vec::new());
/// By place: the answers in turn and how many were given.
static ANSWERS: Mutex<Vec<(u32, Vec<Answer>, usize)>> = Mutex::new(Vec::new());

pub fn regs7(r: &Regs) -> [u32; 7] {
    [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]
}

/// Sets what the stand-ins answer, by place (places without one answer nothing).
pub fn answers(list: &[(u32, &[Answer])]) {
    *ANSWERS.lock().unwrap() = list.iter().map(|&(p, a)| (p, a.to_vec(), 0)).collect();
}

fn begin() {
    CALLED.lock().unwrap().clear();
    for (_, _, given) in ANSWERS.lock().unwrap().iter_mut() {
        *given = 0;
    }
    WENT_FATAL.store(0, Ordering::SeqCst);
}

fn note(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((place, regs7(r)));
    let answer = {
        let mut answers = ANSWERS.lock().unwrap();
        answers
            .iter_mut()
            .find(|(p, a, _)| *p == place && !a.is_empty())
            .map(|(_, a, given)| {
                let n = (*given).min(a.len() - 1);
                *given += 1;
                a[n]
            })
    };
    let Some(a) = answer else { return };
    let set = |to: &mut u32, v: Option<u32>| {
        if let Some(v) = v {
            *to = v;
        }
    };
    set(&mut r.eax, a.eax);
    set(&mut r.ebx, a.ebx);
    set(&mut r.ecx, a.ecx);
    set(&mut r.edx, a.edx);
    set(&mut r.esi, a.esi);
    set(&mut r.edi, a.edi);
    if let Some(c) = a.carry {
        r.set_carry(c);
    }
    for (at, v) in [a.write, a.write2].into_iter().flatten() {
        unsafe { (at as *mut u32).write_unaligned(v) };
    }
}

/// The Rust side's calls in the tests: noted and answered as the stand-ins are.
pub fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

/// A copy of the function at `address` (as IDA shows it), `len` bytes, whose calls at
/// `calls` (offsets of their E8) go to the stand-in of their callee's place in `callees`
/// (addresses as IDA shows them; a callee not there is called as it is), and whose jumps
/// to the fatal reporter (the offsets of their rel32 operands in `fatal`) go to
/// `fatal_landing` (the copy must have begun with `pusha` and have nothing else pushed
/// there).
pub fn copy(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    callees: &'static [usize],
    fatal: &[usize],
) -> Result<RedirectedCopy, &'static str> {
    set_handler(note);
    unsafe {
        let copy = RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            callees
                .iter()
                .position(|&k| rva(k) == c)
                .map(|p| stand_in(p as u32))
        })?;
        for &at in fatal {
            copy.retarget(at, fatal_landing as *const () as usize);
        }
        Ok(copy)
    }
}

/// One case: `prepare`, the copy from `before`, `prepare`, the Rust version from `before`;
/// the registers (and the carry if `carry`), the calls, `regions` and whether it went to the
/// fatal reporter compared. A Rust version that raises a fatal error stands for a copy that
/// went to `fatal_landing` (its registers are then not compared).
#[allow(clippy::too_many_arguments)]
pub fn run_both(
    t: &mut SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    regions: &[Region],
    prepare: &dyn Fn(),
    before: Regs,
    carry: bool,
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
) {
    type Seen = (
        Option<[u32; 7]>,
        Option<bool>,
        Vec<Note>,
        Vec<u8>,
        Option<u32>,
    );
    prepare();
    begin();
    let mut ro = before;
    unsafe { call_game(&mut ro, copy.addr()) };
    let fatal = WENT_FATAL.load(Ordering::SeqCst) == 1;
    let theirs: Seen = (
        (!fatal).then(|| regs7(&ro)),
        (carry && !fatal).then(|| ro.carry()),
        std::mem::take(&mut *CALLED.lock().unwrap()),
        capture(regions),
        fatal.then_some(0),
    );
    prepare();
    begin();
    let outcome = rust(&before);
    let mine: Seen = match &outcome {
        Ok(r) => (
            Some(regs7(r)),
            carry.then(|| r.carry()),
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(regions),
            None,
        ),
        Err(Refused::Fatal(FatalError { .. })) => (
            None,
            None,
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(regions),
            Some(0),
        ),
        Err(Refused::BadInput(_)) => (None, None, Vec::new(), Vec::new(), None),
    };
    t.check(theirs == mine, || {
        let mut at = None;
        let mut start = 0;
        for &(base, len) in regions {
            if theirs.3.len() < start + len || mine.3.len() < start + len {
                break;
            }
            let (a, b) = (&theirs.3[start..start + len], &mine.3[start..start + len]);
            if let Some(k) = a.iter().zip(b).position(|(x, y)| x != y) {
                at = Some((base, k, a[k], b[k]));
                break;
            }
            start += len;
        }
        format!(
            "{what}: original regs {:x?} carry {:?} fatal {} calls {:x?}; rust {:?} regs {:x?} carry {:?} calls {:x?}; state first differs at {at:x?} (region, byte, original, rust)",
            theirs.0,
            theirs.1,
            theirs.4.is_some(),
            theirs.2,
            outcome.as_ref().err(),
            mine.0,
            mine.1,
            mine.2
        )
    });
}
