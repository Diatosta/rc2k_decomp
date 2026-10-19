//! For the self-tests of functions that are mostly calls: a copy of the original whose
//! calls go to stand-ins by place, each noting its place and the registers (which it
//! leaves as they came), and one case run on the copy and on the Rust version with the
//! same stand-ins, the calls, the registers and the memory compared.

use crate::errors::Refused;
use crate::framework::{Regs, SelfTest, call_game, game_addr, rva};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture};
use std::sync::Mutex;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

static CALLED: Mutex<Vec<(u32, [u32; 7])>> = Mutex::new(Vec::new());

fn regs(r: &Regs) -> [u32; 7] {
    [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]
}

pub fn handler(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((place, regs(r)));
}

pub fn take_called() -> Vec<(u32, [u32; 7])> {
    std::mem::take(&mut *CALLED.lock().unwrap())
}

/// The copy (`places`: callee as IDA shows it, its place; other calls left as they are).
pub fn copy(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    places: &'static [(usize, u32)],
) -> Result<RedirectedCopy, &'static str> {
    set_handler(handler);
    unsafe {
        RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            places
                .iter()
                .find(|&&(k, _)| rva(k) == c)
                .map(|&(_, p)| stand_in(p))
        })
    }
}

/// One case.
pub fn both(
    t: &mut SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    entry: Regs,
    fill: &dyn Fn(),
    all: &[Region],
    rust: &dyn Fn(&mut Regs, Part) -> Result<(), Refused>,
) {
    fill();
    take_called();
    let mut ro = entry;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (Some(regs(&ro)), take_called(), capture(all));
    fill();
    let mut g = entry;
    let ok = rust(&mut g, &mut |p, r| {
        handler(p, r);
        Ok(())
    })
    .is_ok();
    let mine = (ok.then(|| regs(&g)), take_called(), capture(all));
    t.check(theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}
