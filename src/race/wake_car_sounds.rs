//! `race::wake_car_sounds` (0x45D255): the cars' sound objects woken at the stage's loading.
//!
//! In:  nothing
//! Out: ebx = the cars' count and edi = the last object woken, when any is; every other
//!      register preserved
//!
//! See `race/sounds.rs`. Callers: `race::load_stage` and two more. Refuses 0x10000 cars or
//! more, a sound object missing. Shadow mode runs the original alone. Self-test: a
//! redirected copy whose call of `race::start_race_sounds` notes itself, the game's
//! objects' flags, `dword_71C3E0` 0 and 1, the cars 0, 1, 5 and 9, put back afterwards.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
