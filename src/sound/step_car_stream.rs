//! `sound::step_car_stream` (0x46F7AD): the car's sound stream goes on to its next part
//! when one has ended.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Only with `dword_7556D0` at 2 and `dword_A8C188` at 1, and while the stream is not
//! playing (`sub_40C4D5`: `dword_506960` is not 1; one other caller): `dword_A8C180` + 1
//! and the part at that place of the list `dword_A8C184`; -1 ends it
//! (`dword_A8C188` = 0), else `sound::start_stream` with it and `dword_755698`. Once a
//! frame on four screens (the car specification, the slideshow, two stage screens).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_read, game_write, rva};

// ---- game data ----

const KIND_RVA: usize = rva(0x7556D0); // dword_7556D0
const ON_RVA: usize = rva(0xA8C188); // dword_A8C188
const STREAM_RVA: usize = rva(0xA8C184); // dword_A8C184
const PLACE_RVA: usize = rva(0xA8C180); // dword_A8C180
const VOLUME_RVA: usize = rva(0x755698); // dword_755698
const PLAYING_RVA: usize = rva(0x506960); // dword_506960
// sub_40C4D5

// ---- Rust version ----

/// `playing` stands for `sub_40C4D5`, `start` for `sound::start_stream` (the part, the
/// volume).
pub unsafe fn step_car_stream(playing: &mut dyn FnMut() -> bool, start: &mut dyn FnMut(u32, u32)) {
    unsafe {
        // Converted: `rc2k_core::sound::race::stream_step`.
        use rc2k_core::sound::race::{StreamStep, stream_step};
        let list: u32 = game_read(STREAM_RVA);
        let (place, part) = match stream_step(
            game_read(KIND_RVA),
            game_read(ON_RVA),
            playing,
            game_read(PLACE_RVA),
            |place| ((list.wrapping_add(place.wrapping_mul(4))) as *const u32).read_unaligned(),
        ) {
            StreamStep::Nothing => return,
            StreamStep::Ended { place } => {
                game_write(PLACE_RVA, place);
                game_write(ON_RVA, 0u32);
                return;
            }
            StreamStep::Start { place, part } => (place, part),
        };
        game_write(PLACE_RVA, place);
        start(part, game_read(VOLUME_RVA));
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe {
        step_car_stream(
            &mut || game_read::<u32>(PLAYING_RVA) == 1,
            &mut crate::sound::start_stream::start_stream,
        )
    };
    Ok(())
}
