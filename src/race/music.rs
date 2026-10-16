//! The race's music at the stage's loading (`race::load_stage`'s "doingamemusic", IDA,
//! 2026-10-15): with `dword_64BB18` set `race::game_music` (tune 6 with `dword_75B108` 1,
//! else tune 5), else `race::game_music_8` (tune 8).

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};

pub const SECOND_KIND_RVA: usize = rva(0x75B108); // dword_75B108

/// A tune sent: 5, 6 or 8.
pub type Tune<'a> = &'a mut dyn FnMut(u32, &Regs) -> Result<(), Refused>;

pub fn game_music(entry: &Regs, tune: Tune) -> Result<(), Refused> {
    let second: u32 = unsafe { game_read(SECOND_KIND_RVA) };
    tune(if second == 1 { 6 } else { 5 }, entry)
}

pub fn game_music_8(entry: &Regs, tune: Tune) -> Result<(), Refused> {
    tune(8, entry)
}

/// In the game: the tune's Rust version, sent and waited for.
pub fn game_tune(tune: u32, entry: &Regs) -> Result<(), Refused> {
    use crate::sound::*;
    let sender = match tune {
        5 => &play_tune_5::SENDER,
        6 => &play_tune_6::SENDER,
        _ => &play_tune_8::SENDER,
    };
    senders::send_in_game(sender, entry);
    Ok(())
}

static CALLED: std::sync::Mutex<Vec<(u32, [u32; 8])>> = std::sync::Mutex::new(Vec::new());

fn regs_of(r: &Regs) -> [u32; 8] {
    [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp, 0]
}

fn note(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((place, regs_of(r)));
}

/// The two's self-test: a copy whose tune calls (by place: `tunes`) note themselves and
/// their registers, against the Rust version; `dword_75B108` 0, 1 and 2, put back.
pub fn check_music(
    t: &mut crate::framework::SelfTest,
    copy: &crate::redirect::RedirectedCopy,
    tunes: &[u32],
    rust: fn(&Regs, Tune) -> Result<(), Refused>,
) {
    use crate::framework::{call_game, game_write};
    use crate::service::harness::{at, capture, restore};
    crate::install::stand_ins::set_handler(note);
    let var = [at(SECOND_KIND_RVA)];
    let saved = capture(&var);
    for second in [0u32, 1, 2] {
        let run = |rust_side: bool| {
            unsafe { game_write(SECOND_KIND_RVA, second) };
            CALLED.lock().unwrap().clear();
            let before = Regs::SENTINEL;
            let ok = if rust_side {
                rust(&before, &mut |tune, r| {
                    let place = tunes.iter().position(|&k| k == tune).unwrap_or(99) as u32;
                    note(place, &mut r.clone());
                    Ok(())
                })
                .is_ok()
            } else {
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                ro.preserved_except(&before, 0)
            };
            (ok, std::mem::take(&mut *CALLED.lock().unwrap()))
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("dword_75B108 {second}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&var, &saved);
}
