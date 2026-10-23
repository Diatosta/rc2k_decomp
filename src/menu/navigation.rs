//! Moving the highlight about a menu: what the four `move_highlight_*` hooks and
//! `handle_keys` share.
//!
//! Each menu item names its neighbours by number at +0x24, +0x28, +0x2C and +0x30 (which key
//! goes with which is in `Direction`). A link of 0x6F means there is none, and 0xFFFFFC1A
//! means "ask the item": its callback for that direction (+0x34 .. +0x40) is called
//! instead. Every move plays the click, `sub_40F059` with eax = 1.
//! The two vertical moves go on along the same link until they reach an item whose +0x48 is
//! 0xFFFFFC1B, so items that cannot be chosen are stepped over. The two horizontal ones take
//! one step and, while `dword_7554EC` is 1, first turn a counter: `dword_75AA7C` goes round
//! 0 .. `dword_75AA78` - 1, and `dword_75A660` and `dword_75AA80` are left at 1 and 0 (right)
//! or -1 and -1 (left). They click only if `dword_7554D0` is not 1.
//!
//! The click, the callbacks and the screenshot are the game's own code. They are reached
//! through `Outside`, so that a shadow comparison can leave them out and a self-test can
//! note them instead.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::item_at::item_at;
use crate::screens::harness::Left;
use crate::service::state::MENU_ITEM_RVA;

// ---- game data ----

const TURNING_RVA: usize = rva(0x7554EC); // dword_7554EC: 1 = left and right turn the counter
const TURN_COUNT_RVA: usize = rva(0x75AA78); // dword_75AA78
const TURN_RVA: usize = rva(0x75AA7C); // dword_75AA7C
const TURN_WAY_RVA: usize = rva(0x75A660); // dword_75A660: 1 after right, -1 after left
const TURN_STEP_RVA: usize = rva(0x75AA80); // dword_75AA80: 0 after right, -1 after left
const QUIET_RVA: usize = rva(0x7554D0); // dword_7554D0: 1 = left and right do not click
pub const CLICK_RVA: usize = rva(0x40F059); // sub_40F059

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The four moves. Which is up and which is down is a reading: the keys are 0x66 and 0x6B.
/// Left and right are keys 0x68 and 0x69, the ones that step a value down and up in the
/// service area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    /// (the link's place in an item, the callback's)
    fn offsets(self) -> (u32, u32) {
        match self {
            Self::Up => (0x24, 0x34),
            Self::Down => (0x28, 0x38),
            Self::Left => (0x2C, 0x3C),
            Self::Right => (0x30, 0x40),
        }
    }
}

// ---- the game's own code these functions call ----

pub trait Outside {
    /// `sub_40F059`: the sound in `regs.eax`.
    fn click(&mut self, regs: &Regs);
    /// An item's callback for a direction.
    fn callback(&mut self, function: u32, regs: &Regs);
    /// `sub_407897`.
    fn screenshot(&mut self, regs: &Regs);
    /// One of the four moves, from `handle_keys`.
    unsafe fn move_highlight(&mut self, direction: Direction, regs: &Regs) -> Result<(), BadInput>;
}

/// The real thing.
pub struct Game;

fn in_game(function: usize, regs: &Regs) {
    let mut g = *regs;
    crate::menu::table_calls::call_any(function as u32, &mut g);
}

impl Outside for Game {
    fn click(&mut self, regs: &Regs) {
        in_game(game_addr(CLICK_RVA), regs);
    }
    fn callback(&mut self, function: u32, regs: &Regs) {
        let mut g = *regs;
        crate::menu::table_calls::call(function, &mut g);
    }
    fn screenshot(&mut self, regs: &Regs) {
        if let Err(e) = crate::gfx::take_screenshot::take_screenshot() {
            unsafe { crate::fatal::raise::refused_in("gfx::take_screenshot", e, regs) };
        }
    }
    unsafe fn move_highlight(&mut self, direction: Direction, regs: &Regs) -> Result<(), BadInput> {
        unsafe { move_highlight(direction, regs, self) }
    }
}

// ---- Rust version of the four moves ----

/// `entry` are the registers at entry: the click and the callback see them, with eax, ebx and
/// edi as the function has them by then.
pub unsafe fn move_highlight(
    direction: Direction,
    entry: &Regs,
    out: &mut dyn Outside,
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::navigation` (`turn`, `link`, `walk`), everything worked
    // out before it is written.
    use rc2k_core::menu::navigation::{Link, Turn, link, turn, walk};
    let (link_at, callback) = direction.offsets();
    let sideways = matches!(direction, Direction::Left | Direction::Right);
    unsafe {
        let highlighted: u32 = game_read(MENU_ITEM_RVA);
        let item = item_at(highlighted)?;
        let turned = (sideways && game_read::<u32>(TURNING_RVA) == 1).then(|| {
            let t = Turn {
                count: game_read(TURN_COUNT_RVA),
                value: game_read(TURN_RVA),
                way: game_read(TURN_WAY_RVA),
                step: game_read(TURN_STEP_RVA),
            };
            turn(t, direction == Direction::Right)
        });
        let next = dword(item + link_at);
        let to = match link(next) {
            Link::To(n) if !sideways => Some(walk(n, |n| {
                let it = item_at(n).map_err(|e| rc2k_core::Invalid(e.0))?;
                Ok((dword(it + link_at), dword(it + 0x48)))
            })?),
            Link::To(n) => Some(n),
            _ => None,
        };
        if let Some(t) = turned {
            game_write(TURN_RVA, t.value);
            game_write(TURN_WAY_RVA, t.way);
            game_write(TURN_STEP_RVA, t.step);
        }
        if link(next) == Link::None {
            return Ok(());
        }
        // `item_at` leaves the item's number in eax.
        let mut regs = Regs {
            eax: highlighted,
            ebx: next,
            edi: item,
            ..*entry
        };
        if !sideways || game_read::<u32>(QUIET_RVA) != 1 {
            regs.eax = 1;
            out.click(&regs);
        }
        match to {
            Some(n) => game_write(MENU_ITEM_RVA, n),
            None => out.callback(dword(item + callback), &regs),
        }
    }
    Ok(())
}

// ---- hook machinery ----

/// The Rust version as `dispatch_outside` runs it.
pub type Run<'a> = &'a dyn Fn(&Regs, &mut dyn Outside) -> Result<Left, Refused>;

/// `dispatch_outside` without its hook (Rust callers; a refusal is the caller's).
pub fn outside_entry(r: &mut Regs, run: Run) -> Result<(), Refused> {
    run(r, &mut Game)?.apply(r);
    Ok(())
}
