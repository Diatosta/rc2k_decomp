//! The controllers' force feedback, the game's side: what the race asks of the six effects of
//! each device record (`types::InputDevice::effects`). A request is a dword at the effect's
//! `pending` (`input::set_effect`: what to do in the low byte, 2 start, 3 set, 4
//! stop; two signed 12-bit numbers above it), which the input thread hands to DirectInput
//! (`input::device_effects`, the platform's side). Nothing here calls DirectInput.
