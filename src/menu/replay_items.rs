//! The review menus' two replay items (`menu::replay_item_a`, `menu::replay_item_b`; what
//! each starts is a reading: the viewer's modes 1 and 3). Both do nothing while
//! `dword_75ACDC` is 1 (a reading: the replay not to be had); see `race/replay_mode.rs`.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};

pub const NO_REPLAY_RVA: usize = rva(0x75ACDC); // dword_75ACDC
const CALLEES: [usize; 4] = [0x41CD75, 0x41100C, 0x40F059, 0x41CD89];

/// A call by place: 0 `race::replay_mode_1`, 1 `system::sleep` (eax), 2
/// `sound::play_menu_sound` (eax), 3 `race::replay_mode_3`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

/// `menu::replay_item_a` (0x47053E): unless `dword_75ACDC` is 1, with a replay there
/// (`dword_5BEC84` 1) `race::replay_mode_1`; else half a second's sleep and the menu's sound
/// 8. The registers it leaves.
pub unsafe fn replay_item_a(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if g(NO_REPLAY_RVA) != 1 && g(crate::race::replay_mode::REPLAY_RVA) == 1 {
        part(0, &mut r)?;
        return Ok(r);
    }
    r.eax = 500;
    part(1, &mut r)?;
    r.eax = 8;
    part(2, &mut r)?;
    Ok(r)
}

/// `menu::replay_item_b` (0x47052F): unless `dword_75ACDC` is 1, `race::replay_mode_3`.
pub unsafe fn replay_item_b(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if g(NO_REPLAY_RVA) != 1 {
        part(3, &mut r)?;
    }
    Ok(r)
}

/// In the game: the replay starts' and the sleep's Rust versions, the menu's sound too.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::replay_mode::{game_part as viewer, replay_mode};
    unsafe {
        match place {
            0 => *r = replay_mode(r, 1, &mut viewer)?,
            1 => crate::system::sleep::sleep(r.eax),
            2 => crate::sound::play_menu_sound::play_menu_sound_in_game(r),
            3 => *r = replay_mode(r, 3, &mut viewer)?,
            _ => crate::calls::call(rva(CALLEES[place as usize]), r),
        }
    }
    Ok(())
}

/// The self-test of either: a copy whose calls go to stand-ins, against the Rust version,
/// each switch.
pub fn test_item(
    hook: &crate::framework::RawHook,
    address: usize,
    len: usize,
    calls: &[usize],
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
) -> (usize, usize) {
    use crate::framework::{SelfTest, game_write};
    use crate::service::harness::{at, capture, restore};
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let places: &'static [(usize, u32)] = Box::leak(
        CALLEES
            .iter()
            .enumerate()
            .map(|(p, &a)| (a, p as u32))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    // cmp dword_75ACDC, 1
    let copy = match crate::race::mud::copy(
        address,
        len,
        &[0x83, 0x3D, 0xDC, 0xAC, 0x75, 0x00, 0x01],
        calls,
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let replay = crate::race::replay_mode::REPLAY_RVA;
    let regions = [at(NO_REPLAY_RVA), at(replay)];
    let saved = capture(&regions);
    for (none, there) in [(0u32, 1u32), (1, 1), (0, 0), (0, 2), (2, 1)] {
        crate::race::mud::run_both(
            &mut t,
            &format!("no replay {none}, replay there {there}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(NO_REPLAY_RVA, none);
                game_write(replay, there);
            },
            rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
