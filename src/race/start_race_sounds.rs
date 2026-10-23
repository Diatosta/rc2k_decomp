//! `race::start_race_sounds` (0x40D592): the race's sounds started.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sounds.rs`. Callers: `race::wake_car_sounds` and 0x41D855. Refuses a sound
//! object missing. Shadow mode runs the original alone. Self-test: a redirected copy whose
//! two calls note themselves, the game's objects' flags, `dword_514F7C` over 0 .. 9 and -1,
//! put back afterwards.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
