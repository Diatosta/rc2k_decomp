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
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

/// Byte `k` of the frame's head (the first car block's start). Asked only for bytes 0 .. 7
/// (the cars' and the four switches', and the extra cars'), inside the head's 8.
fn head_byte(race: &Race, k: usize) -> u32 {
    (race.car_blocks().cars[0].frame_head[k / 4] >> (8 * (k % 4))) & 0xFF
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

/// Where a car's last camera draw frame is kept: by its car (0 .. 3) or by the viewer's choice
/// for a further player's layout (`ViewerBuffers::live_cameras`).
#[derive(Clone, Copy)]
enum LastFrame {
    Car(usize),
    Live(usize),
}

impl LastFrame {
    fn get(self, race: &Race) -> u32 {
        let work = race.replay_work();
        match self {
            LastFrame::Car(c) if c < 2 => work.camera_last_frame[c],
            LastFrame::Car(c) => work.camera_last_frame_extra[c - 2],
            LastFrame::Live(n) => work.camera_last_frame_live[n],
        }
    }

    fn set(self, race: &Race, frame: u32) {
        let mut work = race.replay_work_mut();
        match self {
            LastFrame::Car(c) if c < 2 => work.camera_last_frame[c] = frame,
            LastFrame::Car(c) => work.camera_last_frame_extra[c - 2] = frame,
            LastFrame::Live(n) => work.camera_last_frame_live[n] = frame,
        }
    }
}

/// Car `c`'s camera by the wish of camera byte `byte`.
fn camera_of(race: &Race, c: usize, byte: u32, last_frame: LastFrame) -> Result<(), Refused> {
    let replay = &race.replay;
    let last = last_frame.get(race);
    let wish = wish_of(&race.replay_work(), byte as usize)?;
    let camera_view = race.car_blocks().cars[c].camera_view;
    match wish_step(wish, replay.replay_view().replay_block.get(), last, || {
        camera_view
    }) {
        WishStep::Draw(_) => {
            last_frame.set(race, replay.replay_view().replay_block.get());
            // `Draw` comes only for the wishes that draw: the mode is there.
            let mode = { crate::drive::pick_replay_camera::pick_replay_camera(race, wish) }?;
            let mut blocks = race.car_blocks_mut();
            let car = &mut blocks.cars[c];
            if let Some(mode) = mode {
                car.camera_view = mode;
            }
            car.camera_switch_active = 0;
            car.camera_switch_clock = car.step_clock;
        }
        WishStep::Nothing(_) => {}
        WishStep::Set => {
            {
                let mut blocks = race.car_blocks_mut();
                let car = &mut blocks.cars[c];
                car.camera_blend_view = wish;
                car.camera_view = wish;
                car.camera_switch_active = 0;
                car.camera_switch_clock = car.step_clock;
            }
            last_frame.set(race, 0);
        }
    }
    Ok(())
}

fn steps(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    let replay = &race.replay;
    replay.replay_frames_mut().camera_word_0 = race.car_blocks().cars[0].frame_head[0];
    replay.replay_frames_mut().camera_word_1 = race.car_blocks().cars[0].frame_head[1];
    // A replay with the extra tape also names the cameras of cars 2 and 3, in the head's bytes
    // 6 and 7 (zero in every other replay: no wish).
    let extra = race.viewer_buffers().extra_tape.is_set();
    let cars = if extra { 4 } else { 2 };
    for c in 0..cars {
        let byte = if c < 2 { c } else { c + 4 };
        camera_of(race, c, head_byte(race, byte), LastFrame::Car(c))?;
    }
    // The cars the blocks hold no camera byte for (the fifth to the eighth) follow the choice made
    // for them in the viewer, once made.
    if extra {
        for (n, chosen) in race.viewer_buffers().live_cameras.iter().enumerate() {
            let chosen = chosen.get();
            if chosen != crate::race::viewer_buffers_store::NO_CAMERA {
                camera_of(race, 4 + n, chosen, LastFrame::Live(n))?;
            }
        }
    }
    // A group of players shows every player's HUD as the race did, the recording's own bits; the
    // viewer's other layouts show the HUD pieces of the cars the two views show as the viewer's
    // switches say (the first two cars', and for the pair of the fifth and sixth player theirs).
    // A render takes the viewer's pieces it was given.
    use rc2k_core::race::layouts::Shown;
    let options = shown_hud().unwrap_or([
        shared.view.get().player_options[0].get(),
        shared.view.get().player_options[1].get(),
    ]);
    let shown = crate::race::viewer::layout_shown(race, head_byte(race, 5) as u8);
    let split = matches!(shown, Some(Shown::Group { .. }));
    match shown {
        Some(Shown::Group { .. }) => group_hud(race, options[0]),
        Some(Shown::Pair { first, .. }) if first > 0 => {
            let view = shared.view.get();
            for (view_car, bits) in view.view_cars.iter().zip(options) {
                if let Some(car) = race.car_blocks_mut().cars.get_mut(view_car.get() as usize) {
                    car.option_bits = bits;
                }
            }
        }
        _ => {
            race.car_blocks_mut().cars[0].option_bits = options[0];
            race.car_blocks_mut().cars[1].option_bits = options[1];
        }
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

/// The HUD of the cars of a group the replay holds no HUD pieces for: a car recorded by its
/// motion (another machine's) has none of its own, so each such car's part shows the pieces
/// `own` names (the first player's); the other cars keep the recording's.
fn group_hud(race: &Race, own: u32) {
    let states = race
        .replay
        .v2()
        .lock()
        .ext
        .as_ref()
        .map_or(0, rc2k_core::race::replay_v2::Extension::state_mask);
    let view = race.shared.view.get();
    let first = view.split_first.get() as usize;
    let mut blocks = race.car_blocks_mut();
    let firsts = [view.view_cars[0].get(), view.view_cars[1].get()].map(|car| car as usize);
    for part in 0..(view.views.get() as usize).min(4) {
        let car = firsts.get(part).copied().unwrap_or(first + part);
        if car < 8 && states & (1 << car) != 0 {
            blocks.cars[car].option_bits = own;
        }
    }
}

/// The HUD pieces of the first two cars while the viewer is open (`set_shown_hud`), shown when
/// their HUD is switched on.
static SHOWN_HUD: [AtomicU32; 2] = [const { AtomicU32::new(0) }; 2];
static SHOWN_HUD_SET: AtomicBool = AtomicBool::new(false);

/// The HUD pieces the viewer shows, from its opening to its end (none: the race's own).
pub fn set_shown_hud(bits: Option<[u32; 2]>) {
    if let Some(bits) = bits {
        for (to, bit) in SHOWN_HUD.iter().zip(bits) {
            to.store(bit, Relaxed);
        }
    }
    SHOWN_HUD_SET.store(bits.is_some(), Relaxed);
}

pub fn shown_hud() -> Option<[u32; 2]> {
    SHOWN_HUD_SET
        .load(Relaxed)
        .then(|| [SHOWN_HUD[0].load(Relaxed), SHOWN_HUD[1].load(Relaxed)])
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
        race.car_blocks_mut().cars[0].frame_head = [0x0403_0201, 0x0807_0605];
        for k in 0..8 {
            let old = (k as u32 + 1) as u8;
            assert_eq!(head_byte(race, k), u32::from(old), "byte {k}");
        }
    }
}
