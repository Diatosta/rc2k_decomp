//! `race::set_up_credits_race` (0x463711): the credits' race (the demonstration between
//! the films) set up: a stage drawn from the 36 of `dword_71C3E4` (`math::next_random`
//! x 36, the high dword, held to 0 .. 0x23 by the sign) into `g_stageNumber`, a car kind
//! from the 23 of `dword_71C474` (x 23, 0 .. 0x16) into `g_raceCarKinds`;
//! `dword_623854` 0, `dword_62389C` 3, and 72 more stores (`WRITES`: the first car
//! block's settings, the race's switches, -1 into +0x298).
//!
//! In:  nothing
//! Out: eax 3, ebx 0x17, edx the car kind's index; the rest kept; no flags read (a call
//!      follows)
//!
//! Caller: `race::credits_race`, which calls the Rust version directly. It begins with a
//! call: a private copy (its two calls relocated) with a hand-made patch.
//! Refuses nothing. Self-test: the original and the Rust version on the game's
//! variables (put back), from twelve states of the generator.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::math::next_random::next_random;

const STAGES: usize = 0x71C3E4; // dword_71C3E4, `RACE_STAGES` stages
const KINDS: usize = 0x71C474; // dword_71C474, `RACE_KINDS` car kinds
const STAGE: usize = 0x623850; // g_stageNumber
const CAR_KINDS: usize = 0x623858; // g_raceCarKinds

/// The stores after the two draws, in order (address as IDA shows it, value).
pub const WRITES: [(usize, u32); 74] = [
    (0x623854, 0),
    (0x62389C, 3),
    (0x70FE94, 1),
    (0x70FA70, 8),
    (0x70F414, 2),
    (0x70F420, 0x80),
    (0x70F41C, 0x80),
    (0x70F40C, 0x80),
    (0x70F428, 0x80),
    (0x6235FC, 3),
    (0x623878, 1),
    (0x623898, 0),
    (0x63DB90, 1),
    (0x63DB94, 1),
    (0x6238A8, 1),
    (0x6238AC, 1),
    (0x6238B0, 1),
    (0x6238B4, 0),
    (0x6238B8, 1),
    (0x621890, 1),
    (0x6238CC, 0),
    (0x6238D0, 0),
    (0x70F6A4, 0),
    (0x70F6A8, 0),
    (0x70F6FC, 0),
    (0x70F6AC, 0),
    (0x70F6B0, 0),
    (0x70F71C, 0),
    (0x70F6B4, 0),
    (0x70F6B8, 0),
    (0x70F730, 0),
    (0x70F750, 0),
    (0x70F73C, 0),
    (0x70F718, 0),
    (0x70F6E0, 0),
    (0x70F6F8, 0),
    (0x70F6BC, 0),
    (0x70F6C4, 0),
    (0x70F6C8, 0),
    (0x70F6CC, 0),
    (0x70F6D0, 0),
    (0x70F6D4, 0),
    (0x70F6D8, 0),
    (0x70F6DC, 0),
    (0x70F6E4, 0),
    (0x70F6E8, 0),
    (0x70F6EC, 0),
    (0x70F6F0, 0),
    (0x70F6F4, 0),
    (0x70F700, 0),
    (0x70F704, 0),
    (0x70F708, 0),
    (0x70F70C, 0),
    (0x70F710, 0),
    (0x70F714, 0),
    (0x70F720, 0),
    (0x70F724, 0),
    (0x70F728, 0),
    (0x70F72C, 0),
    (0x70F734, 0),
    (0x70F738, 0),
    (0x70F740, 0),
    (0x70F744, 0),
    (0x70F748, 0),
    (0x70F74C, 0),
    (0x70F450, 0),
    (0x70F454, 0),
    (0x70F458, 0),
    (0x70F45C, 0),
    (0x70F460, 0),
    (0x70F464, 0),
    (0x70F468, 0),
    (0x70F46C, 0),
    (0x70F684, 0xFFFF_FFFF),
];

/// A draw (`math::next_random`) as a place among n.
fn drawn_index(n: u32) -> u32 {
    rc2k_core::race::credits::drawn_place(unsafe { next_random() }, n)
}

/// The Rust version. (The original leaves eax 3, ebx 0x17 and edx the car kind drawn;
/// nothing reads them.)
pub fn set_up_credits_race() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::credits::drawn_place`.
    use rc2k_core::race::credits::{RACE_KINDS, RACE_STAGES};
    unsafe {
        let stage = drawn_index(RACE_STAGES);
        game_write(
            rva(STAGE),
            game_read::<u32>(rva(STAGES) + 4 * stage as usize),
        );
        let kind = drawn_index(RACE_KINDS);
        game_write(
            rva(CAR_KINDS),
            game_read::<u32>(rva(KINDS) + 4 * kind as usize),
        );
        for (at, v) in WRITES {
            game_write(rva(at), v);
        }
        Ok(())
    }
}
