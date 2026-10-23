//! The race sounds' turn (stream B, 2026-10-18): `sound::race_turn` (0x40D5F3) and the
//! trees under it, which work on the eight cars' sound objects (`g_raceSoundObjects`,
//! 0x4A8 bytes each), the shared spots' voices (`unk_5168C0`) and one request
//! (`unk_5124BC`, ten dwords, see `sound::play_placed_sound`) they send each sound with.
//! This file: their calls by place, the place being the callee's IDA address: in the game
//! the Rust versions, in the tests stand-ins that note each call with its registers (and
//! the request when esi points at it) and change nothing but eax for the three that give
//! a number; and copies of the originals whose calls (and jumps out) go to them.
//!
//! Shadow mode runs the originals that send sounds alone (a sound must not be sent
//! twice); those that only work out state run both, compared (`dispatch_turn`).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::service::harness::Region;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;
/// A Rust version: the registers as the original leaves them.
pub type Rust = fn(&mut Regs, Part) -> Result<(), Refused>;

pub const PLAY: u32 = 0x409B36; // sound::play_sound
pub const STOP: u32 = 0x40F1D1; // sound::stop_sound
pub const PLACED: u32 = 0x40F11C; // sound::play_placed_sound
pub const RANDOM: u32 = 0x4018B4; // math::next_random
pub const BELOW: u32 = 0x4018A2; // math::random_below
pub const TICK: u32 = 0x411024; // system::tick_count
/// `sound::step_race_clock`: carry set, no turn due.
pub const STEP_CLOCK: u32 = 0x40F20E;
/// `input::update_effects`: the force feedback's requests (the game's side; DirectInput
/// gets them on the input thread).
pub const EFFECTS: u32 = 0x40F2C0;

/// unk_5124BC: the request the turn's sounds go with.
pub const REQUEST: usize = 0x5124BC;
pub const REQUEST_LEN: usize = 0x28;
/// dword_5124E8: the time of the sounds' last turn (`sound/race_clock.rs`).
pub const CLOCK: usize = 0x5124E8;
/// g_raceSoundObjects: eight pointers.
pub const OBJECTS: usize = 0x512A10;
/// dword_514F7C: the sound objects in use.
pub const COUNT: usize = 0x514F7C;
pub const OBJECT_LEN: usize = 0x4A8;
/// dword_5183CC: the scratch dword the x87 code goes through.
pub const SCRATCH: usize = 0x5183CC;

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::sound::play_sound::play_sound_in_game;
    match place {
        PLAY => play_sound_in_game(r),
        STOP => {
            if let Some(with) = unsafe { crate::sound::stop_sound::stop_sound(r) }? {
                play_sound_in_game(&with);
            }
        }
        PLACED => unsafe {
            let mode = game_read(crate::sound::play_placed_sound::MODE_RVA);
            let samples = game_addr(crate::sound::play_placed_sound::SAMPLES_RVA);
            crate::sound::play_placed_sound::play_placed_sound(r, mode, samples, &mut |w| {
                play_sound_in_game(w)
            })?;
        },
        RANDOM => r.eax = unsafe { crate::math::next_random::next_random() },
        BELOW => r.eax = unsafe { crate::math::random_below::random_below(r.eax) }?,
        TICK => r.eax = crate::system::tick_count::tick_count(),
        EFFECTS => {
            crate::input::update_effects::update_effects(r, &mut crate::input::effects::game_part)?
        }
        0x40E59E => crate::sound::play_timed_slot::play_timed_slot_in(r, &mut game_part)?,
        0x40D8E2 => crate::sound::car_engine_state::car_engine_state_in(r, &mut game_part)?,
        0x40DD19 => crate::sound::revs_sound::revs_sound_in(r, &mut game_part)?,
        0x40DD96 => crate::sound::throttle_sound::throttle_sound_in(r, &mut game_part)?,
        0x40DE09 => crate::sound::speed_sound::speed_sound_in(r, &mut game_part)?,
        0x40DE9B => crate::sound::surface_sounds::surface_sounds_in(r, &mut game_part)?,
        0x40E068 => crate::sound::surface_sound::surface_sound_in(r, &mut game_part)?,
        0x40E0F6 => crate::sound::car_slot_sounds::car_slot_sounds_in(r, &mut game_part)?,
        0x40E182 => crate::sound::car_hit_sound::car_hit_sound_in(r, &mut game_part)?,
        0x40E1E5 => crate::sound::car_cue_sound::car_cue_sound_in(r, &mut game_part)?,
        0x40E24F => crate::sound::car_gear_sounds::car_gear_sounds_in(r, &mut game_part)?,
        0x40D893 => crate::sound::car_sounds::car_sounds_in(r, &mut game_part)?,
        0x40D75A => crate::sound::mark_car_voices::mark_car_voices_in(r, &mut game_part)?,
        0x40D7AB => crate::sound::car_sounds_all::car_sounds_all_in(r, &mut game_part)?,
        0x40ED13 => crate::sound::play_cue_queues::play_cue_queues_in(r, &mut game_part)?,
        0x40ED4A => crate::sound::play_cue_queue::play_cue_queue_in(r, &mut game_part)?,
        0x40EE32 => crate::sound::drop_cue::drop_cue_in(r, &mut game_part)?,
        0x40E2A9 => crate::sound::stage_sounds::stage_sounds_in(r, &mut game_part)?,
        0x40E611 => crate::sound::spot_sounds::spot_sounds_in(r, &mut game_part)?,
        0x40E642 => crate::sound::match_spot_voices::match_spot_voices_in(r, &mut game_part)?,
        0x40E6F4 => crate::sound::free_spot_voices::free_spot_voices_in(r, &mut game_part)?,
        0x40E733 => crate::sound::spot_voice_distances::spot_voice_distances_in(r, &mut game_part)?,
        0x40E7CA => {
            crate::sound::merge_spot_voices_all::merge_spot_voices_all_in(r, &mut game_part)?
        }
        0x40E809 => crate::sound::merge_spot_voices::merge_spot_voices_in(r, &mut game_part)?,
        0x4633DE => crate::math::nearest_on_segment::nearest_on_segment_in(r, &mut game_part)?,
        0x40E97D => crate::sound::thin_spot_voices_all::thin_spot_voices_all_in(r, &mut game_part)?,
        0x40E9F8 => crate::sound::thin_spot_voices::thin_spot_voices_in(r, &mut game_part)?,
        0x40EAF3 => {
            crate::sound::exclusive_spot_voices::exclusive_spot_voices_in(r, &mut game_part)?
        }
        0x40EB44 => crate::sound::repeat_spot_voices::repeat_spot_voices_in(r, &mut game_part)?,
        0x40EB9E => crate::sound::play_spot_voices::play_spot_voices_in(r, &mut game_part)?,
        0x409BE0 => crate::sound::send_end_of_turn::send_end_of_turn_in(r, &mut game_part)?,
        0x40F257 => crate::input::step_effects_clock::step_effects_clock_in(r, &mut game_part)?,
        0x40D5F3 => crate::sound::race_turn::race_turn_in(r, &mut game_part)?,
        0x421BCF => crate::sound::race_turn_if::race_turn_if_in(r, &mut game_part)?,
        // sound::send_command with command 0xA (`send_end_of_turn`'s one call).
        0x409C45 => crate::sound::send_end_of_turn::send_end_of_turn_in_game(r),
        STEP_CLOCK => {
            let due = unsafe {
                crate::sound::race_clock::step_race_clock(crate::system::tick_count::tick_count())
            };
            r.set_carry(!due);
            r.set_zero(false);
        }
        // The quit jump: the original, as it is (held for the owner's decision).
        crate::sound::race_turn::QUIT => crate::sound::race_turn::quit_in_game(r),
        _ => return Err(BadInput("a race sound call with no version").into()),
    }
    Ok(())
}

// ---- the dispatch ----

/// `dispatch_turn` / `dispatch_turn_floats` without the hook (no shadow mode; a refusal is
/// the caller's): the Rust version with the game's parts, its registers but esp and the
/// flags given back.
pub fn turn_entry(r: &mut Regs, rust: Rust) -> Result<(), Refused> {
    let mut g = *r;
    rust(&mut g, &mut game_part)?;
    *r = Regs {
        esp: r.esp,
        eflags: r.eflags,
        ..g
    };
    Ok(())
}

// ---- the tests' side ----

/// The request and the scratch dword, which every one of these may write.
pub fn common_regions() -> Vec<Region> {
    vec![
        (game_addr(rva(REQUEST)) as u32, REQUEST_LEN),
        (game_addr(rva(SCRATCH)) as u32, 4),
    ]
}

/// Everything the turn's trees write but the sounds they send: the objects in use, the
/// request, the scratch dword and the variables between (for `guard`).
pub fn turn_regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut all: Vec<Region> = (0..8usize)
        .map(|n| unsafe { game_read::<u32>(rva(OBJECTS + 4 * n)) })
        .filter(|&o| o != 0)
        .map(|o| (o, OBJECT_LEN))
        .collect();
    all.extend(common_regions());
    all.extend([
        // dword_514F80, dword_514F84, flt_514F88
        (a(0x514F80), 12),
        // flt_51638C .. flt_516398
        (a(0x51638C), 0x10),
        // dword_4F66F1, dword_4F66F5 (unaligned)
        (a(0x4F66F1), 8),
    ]);
    all
}
