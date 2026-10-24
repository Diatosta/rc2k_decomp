//! For the menu set-ups and item actions ported on 2026-10-19 (agent E): the calls a Rust
//! version makes go through a `Part` by place (its callee's place in its list), so that a
//! self-test can run a copy of the original whose calls go to stand-ins of those places and
//! the Rust version on the same stand-ins' notes. Each call is noted with the seven
//! registers; what a stand-in answers (registers, the carry, a game dword written) is set
//! per place, in turn (the last answer stays).
