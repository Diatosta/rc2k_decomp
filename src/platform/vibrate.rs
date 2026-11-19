//! The phone's vibration for the game's rumble. The touch screen's virtual gamepad has rumble
//! motors in SDL's eyes, so it is a force feedback device like a pad: the controller page's
//! force feedback strength and the advanced page's force feedback switch apply to it as to any
//! pad, and what the game's effects add up to reaches `rumble` as two motor levels. They are
//! mixed down to the phone's one motor as short pulses (`rc2k_core::input::rumble::
//! phone_pulse`), at most one every `PHONE_PULSE_GAP_MS`.

use rc2k_core::input::rumble::{PHONE_PULSE_GAP_MS, phone_pulse};
use std::sync::Mutex;
use std::time::Instant;

/// When the phone last pulsed.
static LAST_PULSE: Mutex<Option<Instant>> = Mutex::new(None);

/// The game's rumble for the touch screen's pad at motor levels `heavy` and `light` (0 .. 65535;
/// both 0 when it stops): the phone pulses for a strong enough one, not more often than the
/// gap allows, and goes still when the rumble stops.
pub fn rumble(heavy: u16, light: u16) {
    let Some(pulse) = phone_pulse(heavy, light) else {
        if heavy == 0 && light == 0 {
            crate::platform::ffi::phone_vibrator::stop();
        }
        return;
    };
    let now = Instant::now();
    let mut last = LAST_PULSE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if last.is_some_and(|at| now.duration_since(at).as_millis() < PHONE_PULSE_GAP_MS) {
        return;
    }
    *last = Some(now);
    crate::platform::ffi::phone_vibrator::pulse(pulse.strength, pulse.ms);
}
