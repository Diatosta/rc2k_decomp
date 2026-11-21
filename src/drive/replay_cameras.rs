//! The two cars' replay cameras from the frame's head (the first car block's first dwords). For
//! each car, the head's byte picks a wish (`replay_camera_wishes`); 0x23 .. 0x25 at least three
//! frames after the last draw a camera mode (`drive::pick_replay_camera`), another wish not
//! already set is taken as it is. Then the option bits of both come from `player_options` and
//! the head's bytes 2 .. 5 become the view switches. Refuses what `pick_replay_camera` refuses
//! (everything written put back).

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::replay_work_store::ReplayWork;
use rc2k_core::drive::replay_views::{WishStep, wish_step};

/// Byte `k` of the frame's head (the first car block's start). Asked only for bytes 0 .. 7
/// (the cars' and the four switches', and the extra cars'), inside the head's 8.
fn head_byte(race: &Race, k: usize) -> u32 {
    (race.car_blocks().cars[0].frame_head[k / 4].get() >> (8 * (k % 4))) & 0xFF
}

/// `replay_camera_wishes[head byte]`: the table holds 47; the two dwords that follow it in
/// the replay's work, the last draw frames (`camera_last_frame`), answer for the bytes 47
/// and 48; any larger byte is refused (a damaged replay file gives any byte).
fn wish_of(replay_work: &ReplayWork, byte: usize) -> Result<u32, BadInput> {
    replay_work
        .replay_camera_wishes
        .iter()
        .chain(&replay_work.camera_last_frame)
        .nth(byte)
        .copied()
        .ok_or(BadInput("a replay camera wish past the replay work"))
}

/// Car `c`'s last camera draw frame set to `frame`.
fn set_last_frame(race: &Race, c: usize, frame: u32) {
    let mut work = race.replay_work_mut();
    if c < 2 {
        work.camera_last_frame[c] = frame;
    } else {
        work.camera_last_frame_extra[c - 2] = frame;
    }
}

fn steps(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    let replay = &race.replay;
    replay.replay_frames_mut().camera_word_0 = race.car_blocks().cars[0].frame_head[0].get();
    replay.replay_frames_mut().camera_word_1 = race.car_blocks().cars[0].frame_head[1].get();
    // A replay with the extra tape also names the cameras of cars 2 and 3, in the head's bytes
    // 6 and 7 (zero in every other replay: no wish).
    let cars = if race.viewer_buffers().extra_tape.is_set() {
        4
    } else {
        2
    };
    for c in 0..cars {
        let car = &race.car_blocks().cars[c];
        let byte = if c < 2 { c } else { c + 4 };
        let last = {
            let work = race.replay_work();
            if c < 2 {
                work.camera_last_frame[c]
            } else {
                work.camera_last_frame_extra[c - 2]
            }
        };
        let wish = wish_of(&race.replay_work(), head_byte(race, byte) as usize)?;
        match wish_step(wish, replay.replay_view().replay_block.get(), last, || {
            car.camera_view.get()
        }) {
            WishStep::Draw(_) => {
                set_last_frame(race, c, replay.replay_view().replay_block.get());
                // `Draw` comes only for the wishes that draw: the mode is there.
                if let Some(mode) =
                    { crate::drive::pick_replay_camera::pick_replay_camera(race, wish) }?
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
                set_last_frame(race, c, 0);
            }
        }
    }
    // The split-screen layout shows every player's HUD as the race did, the recording's own bits
    // (the viewer's other layouts show the first two cars' as the viewer's switches say).
    // A render takes the viewer's pieces it was given: its own replay was not set up from the
    // menus, which is where the players' settings are taken in.
    let split = head_byte(race, 5) == rc2k_core::race::replay_v2::SPLIT_LAYOUT;
    if !split {
        let options = crate::video::hud_bits().unwrap_or([
            shared.view.get().player_options[0].get(),
            shared.view.get().player_options[1].get(),
        ]);
        race.car_blocks().cars[0].option_bits.set(options[0]);
        race.car_blocks().cars[1].option_bits.set(options[1]);
    }
    // The head's bytes 2 .. 5 into the view switches.
    for (k, to) in [
        &shared.view.get().view_on[0],
        &shared.view.get().view_on[1],
        &shared.view.get().after_credits,
        &shared.view.get().view_switch,
    ]
    .into_iter()
    .enumerate()
    {
        to.set(head_byte(race, 2 + k));
    }
    // Every part of the split screen shows its HUD, as in the race.
    if split {
        shared.view.get().view_on[0].set(1);
        shared.view.get().view_on[1].set(1);
    }
    Ok(())
}

pub fn replay_cameras(race: &Race) -> Result<(), Refused> {
    steps(race)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wish_stays_inside_the_replay_work() {
        assert!(wish_of(&crate::test_support::own::race().replay_work(), 46).is_ok());
        // The two dwords after the table are the replay work's own.
        assert!(wish_of(&crate::test_support::own::race().replay_work(), 48).is_ok());
        assert!(wish_of(&crate::test_support::own::race().replay_work(), 255).is_err());
    }

    /// Each byte of the head, as the byte-offset read gave it.
    #[test]
    fn head_bytes_as_the_old_read() {
        let race = crate::test_support::own::race();
        let head = &race.car_blocks().cars[0].frame_head;
        head[0].set(0x0403_0201);
        head[1].set(0x0807_0605);
        for k in 0..8 {
            let old = crate::memory::cells::load_value::<u8>(head, k).map(u32::from);
            assert_eq!(Ok(head_byte(race, k)), old, "byte {k}");
        }
    }
}
