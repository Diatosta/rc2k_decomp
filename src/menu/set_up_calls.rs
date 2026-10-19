//! The Rust versions of the set-ups the Rust screen loops call (`Step::Call`, `env.call`),
//! for `screen_loop::Real` (agent E, 2026-10-19): `call` runs the one at `function` (an
//! RVA) and says whether it did; a refusal leaves the registers as they were and says no,
//! so that the call goes to the hook, which deals with it (the original, or the fatal
//! error raised).

use crate::errors::Refused;
use crate::framework::{Regs, rva};
use crate::screens::harness::Left;

fn left(r: &mut Regs, out: Result<Left, Refused>) -> bool {
    match out {
        Ok(l) => {
            l.apply(r);
            true
        }
        Err(_) => false,
    }
}

fn regs(r: &mut Regs, out: Result<Regs, Refused>) -> bool {
    match out {
        Ok(left) => {
            *r = Regs { esp: r.esp, ..left };
            true
        }
        Err(_) => false,
    }
}

/// Runs the Rust version of the set-up at `function`, if it is one of these.
pub unsafe fn call(function: usize, r: &mut Regs) -> bool {
    use crate::menu::{controller_items as c, results_setups as s, stage_view_steps as v};
    let entry = *r;
    unsafe {
        match function {
            f if f == rva(0x48340B) => left(r, s::rank_stage_times(&entry)),
            f if f == rva(0x48356F) => left(r, s::rank_totals(&entry)),
            f if f == rva(0x485CBB) => left(r, s::score_four_players(&entry)),
            f if f == rva(0x485B0E) => left(r, s::score_two_players(&entry)),
            f if f == rva(0x470F9F) => left(r, v::style_stage_item(false)),
            f if f == rva(0x470FC3) => left(r, v::style_stage_item(true)),
            f if f == rva(0x480032) => left(r, v::count_stage_pages().map(|_| Left::nothing())),
            f if f == rva(0x47F437) => regs(r, v::play_stage_speech(&entry, &mut v::speech_part)),
            f if f == rva(0x47F40F) => regs(r, v::play_rally_speech(&entry, &mut v::speech_part)),
            f if f == rva(0x4684BD) => {
                regs(r, v::load_welcome_picture(&entry, &mut v::picture_part))
            }
            f if f == rva(0x46B47C) => left(r, c::apply_controller().map(|_| Left::nothing())),
            f if f == rva(0x46B4B4) => {
                c::release_controller();
                true
            }
            f if f == rva(0x480CBF) => regs(
                r,
                crate::menu::check_cheat_name::check_cheat_name(
                    &entry,
                    &mut crate::menu::check_cheat_name::game_part,
                ),
            ),
            _ => false,
        }
    }
}
