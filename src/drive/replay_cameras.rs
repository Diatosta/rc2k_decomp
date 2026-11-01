//! `drive::replay_cameras` (0x460949): the two cars' replay cameras from the frame's head
//! (the first car block, `g_carBlockStore`: its first two dwords into `dword_6EF7D0`,
//! `6EF7D4`). For each car c (block c, its byte c of the head): the wish
//! `dword_719F14[byte]`; 0x23 .. 0x25 at least three frames after the last
//! (`dword_719FD0[c]`, frame `dword_5C2530`): a mode drawn (`drive::pick_replay_camera`)
//! into +0x684, +0x694 0, +0x690 the block's +8; another wish not already +0x684: it into
//! +0x688 and +0x684, the same two, the last frame 0. Then the option bits (+0x220) of
//! both from `dword_63DB7C`, `63DB80`, and the head's bytes 2 .. 5 into `dword_63DB74`,
//! `63DB78`, `63DB84`, `63DB88`.
//!
//! In:  nothing
//! Out: eax the head's byte 5, esi the first block, ebx and edx as the draws leave them;
//!      the rest kept; no flags read
//!
//! Callers: `drive::play_replay_frame`, `sub_46062E`. Refuses what `pick_replay_camera`
//! refuses (everything written put back).

use crate::drive::replay_frames_store::REPLAY_FRAMES;
use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::view_store::RACE_VIEW;
use crate::race::viewer_state_store::REPLAY_VIEW;
use rc2k_core::drive::replay_views::{WishStep, wish_step};

/// Byte `k` of the frame's head (the first car block's start, `g_carBlockStore`).
fn head_byte(k: usize) -> Result<u32, BadInput> {
    GamePtr::to(&CAR_BLOCKS.cars[0].unknown_0)
        .offset(k as u32)
        .read::<u8>("a car's block")
        .map(u32::from)
}

/// `dword_719F14[head byte]`: the table holds 47; a larger byte reads on into the
/// variables after it, as the original does, and is refused past the replay's work (a
/// damaged replay file gives any byte).
fn wish_of(byte: usize) -> Result<u32, BadInput> {
    crate::session::read_on_in(
        &*REPLAY_WORK,
        &REPLAY_WORK.replay_camera_wishes,
        byte,
        "the replay camera wishes",
    )
}

// Converted: `rc2k_core::drive::replay_views::wish_step`.
fn steps() -> Result<(), Refused> {
    REPLAY_FRAMES
        .camera_word_0
        .set(CAR_BLOCKS.cars[0].unknown_0[0].bits());
    REPLAY_FRAMES
        .camera_word_1
        .set(CAR_BLOCKS.cars[0].unknown_0[1].bits());
    for c in 0..2usize {
        let car = &CAR_BLOCKS.cars[c];
        let wish = wish_of(head_byte(c)? as usize)?;
        match wish_step(
            wish,
            REPLAY_VIEW.replay_block.get(),
            REPLAY_WORK.camera_last_frame[c].get(),
            || car.camera_view.get(),
        ) {
            WishStep::Draw(_) => {
                REPLAY_WORK.camera_last_frame[c].set(REPLAY_VIEW.replay_block.get());
                // `Draw` comes only for the wishes that draw: the mode is there.
                if let Some(mode) = { crate::drive::pick_replay_camera::pick_replay_camera(wish) }?
                {
                    car.camera_view.set(mode);
                }
                car.camera_switch_active.set(0);
                car.camera_switch_clock.set(car.step_clock.get());
            }
            WishStep::Nothing(_) => {}
            WishStep::Set => {
                car.camera_blend_view.set(wish);
                car.camera_view.set(wish);
                car.camera_view.set(wish);
                car.camera_switch_active.set(0);
                car.camera_switch_clock.set(car.step_clock.get());
                REPLAY_WORK.camera_last_frame[c].set(0);
            }
        }
    }
    CAR_BLOCKS.cars[0]
        .option_bits
        .set_bits(RACE_VIEW.player_options[0].get());
    CAR_BLOCKS.cars[1]
        .option_bits
        .set_bits(RACE_VIEW.player_options[1].get());
    // The head's bytes 2 .. 5 into `dword_63DB74`, `63DB78`, `63DB84`, `63DB88`.
    for (k, to) in [
        &RACE_VIEW.view_on[0],
        &RACE_VIEW.view_on[1],
        &RACE_VIEW.after_credits,
        &RACE_VIEW.view_switch,
    ]
    .into_iter()
    .enumerate()
    {
        to.set(head_byte(2 + k)?);
    }
    Ok(())
}

/// The Rust version.
pub fn replay_cameras() -> Result<(), Refused> {
    steps()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wish_stays_inside_the_replay_work() {
        assert!(wish_of(46).is_ok());
        // The two dwords after the table are the replay work's own.
        assert!(wish_of(48).is_ok());
        assert!(wish_of(255).is_err());
    }
}
