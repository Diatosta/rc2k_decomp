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
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, rva, shadow,
};
use crate::input::take_key::{notes_taken_keys, noting_taken_key, replaying_key};
use crate::menu::item_at::{CURRENT_MENU_RVA, item_at};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::MENU_ITEM_RVA;
use std::sync::atomic::{AtomicU32, Ordering};

// ---- game data ----

const TURNING_RVA: usize = rva(0x7554EC); // dword_7554EC: 1 = left and right turn the counter
const TURN_COUNT_RVA: usize = rva(0x75AA78); // dword_75AA78
const TURN_RVA: usize = rva(0x75AA7C); // dword_75AA7C
const TURN_WAY_RVA: usize = rva(0x75A660); // dword_75A660: 1 after right, -1 after left
const TURN_STEP_RVA: usize = rva(0x75AA80); // dword_75AA80: 0 after right, -1 after left
const QUIET_RVA: usize = rva(0x7554D0); // dword_7554D0: 1 = left and right do not click
pub const CLICK_RVA: usize = rva(0x40F059); // sub_40F059
pub const SCREENSHOT_RVA: usize = rva(0x407897); // sub_407897

const NO_LINK: u32 = 0x6F;
const ASK_ITEM: u32 = 0xFFFF_FC1A;
const CAN_BE_CHOSEN: u32 = 0xFFFF_FC1B;

/// What the four moves may write themselves.
pub fn move_regions() -> [Region; 4] {
    [
        at(MENU_ITEM_RVA),
        at(TURN_RVA),
        at(TURN_WAY_RVA),
        at(TURN_STEP_RVA),
    ]
}

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
    unsafe { call_game(&mut g, function) };
}

impl Outside for Game {
    fn click(&mut self, regs: &Regs) {
        in_game(game_addr(CLICK_RVA), regs);
    }
    fn callback(&mut self, function: u32, regs: &Regs) {
        in_game(function as usize, regs);
    }
    fn screenshot(&mut self, regs: &Regs) {
        // `gfx::take_screenshot`'s Rust version (what it refuses through its hook).
        let mut r = *regs;
        let _ = crate::race::loadlevel::or_in_game(0x407897, &mut r, |r| {
            use crate::gfx::take_screenshot::{game_part, take_screenshot};
            *r = take_screenshot(r, &mut game_part)?;
            Ok(())
        });
    }
    unsafe fn move_highlight(&mut self, direction: Direction, regs: &Regs) -> Result<(), BadInput> {
        unsafe { move_highlight(direction, regs, self) }
    }
}

/// Nothing happens outside; `left_out` says whether something was skipped that could have
/// changed the game's variables (a callback or a screenshot, not a click).
#[derive(Default)]
pub struct Silent {
    pub left_out: bool,
}

impl Outside for Silent {
    fn click(&mut self, _: &Regs) {}
    fn callback(&mut self, _: u32, _: &Regs) {
        self.left_out = true;
    }
    fn screenshot(&mut self, _: &Regs) {
        self.left_out = true;
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

/// The `dispatch` of a hook whose function calls the game's own code through `Outside`. In
/// shadow mode the original runs first, for real; then the variables are put back and the
/// Rust version runs with `Silent` (and, if `keyed`, with the key the original took); the
/// results are compared unless the Rust version had to leave out a callback or a screenshot,
/// which may have changed anything; and the original's results are put in place.
pub fn dispatch_outside(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    regions: &[Region],
    run: Run,
    keyed: bool,
) {
    hook.count_call();
    if !shadow() {
        match run(r, &mut Game) {
            Ok(left) => left.apply(r),
            Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
        }
        return;
    }
    if keyed && !notes_taken_keys() {
        // With the key hook off the key cannot be replayed.
        unsafe { call_original(r) };
        return;
    }
    let before = capture(regions);
    let mut o = *r;
    let key = if keyed {
        noting_taken_key(|| unsafe { call_original(&mut o) }).1
    } else {
        unsafe { call_original(&mut o) };
        None
    };
    let theirs = capture(regions);

    restore(regions, &before);
    let mut silent = Silent::default();
    let result = if keyed {
        replaying_key(key.unwrap_or(0), || run(r, &mut silent))
    } else {
        run(r, &mut silent)
    };
    let mine = capture(regions);
    restore(regions, &theirs);

    match result {
        Err(Refused::Fatal(error)) => {
            crate::screens::harness::mismatch_unless_disturbed(hook, || {
                format!(
                    "the Rust version reported fatal error {:#x} {:?}; the original returned",
                    error.code, error.message
                )
            })
        }
        Ok(left) if !silent.left_out && (theirs != mine || !left.matches(r, &o)) => {
            crate::screens::harness::mismatch_unless_disturbed(hook, || {
                format!(
                    "key={key:x?} before {before:x?}: original {theirs:x?} eax={:#x}; rust {mine:x?} {left:?}",
                    o.eax
                )
            })
        }
        _ => {}
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

// ---- self-test machinery: stand-ins that note how they were called ----

const NOTE_SLOTS: usize = 16;
static NOTES: [AtomicU32; NOTE_SLOTS * 4] = [const { AtomicU32::new(0) }; NOTE_SLOTS * 4];
static NOTE_COUNT: AtomicU32 = AtomicU32::new(0);
pub static TEST_KEY: AtomicU32 = AtomicU32::new(0);

pub const CLICK: u32 = 1;
pub const CALLBACK: u32 = 2;
pub const SCREENSHOT: u32 = 3;
pub const MOVE: u32 = 4; // + the direction

/// A stand-in for a piece of the game's code: notes its number and eax, ebx and edi.
macro_rules! noting_stand_in {
    ($name:ident, $id:expr) => {
        #[unsafe(naked)]
        pub unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "push ecx",
                "mov ecx, [{count}]",
                "and ecx, 15",
                "shl ecx, 4",
                "mov dword ptr [{notes} + ecx], {id}",
                "mov [{notes} + ecx + 4], eax",
                "mov [{notes} + ecx + 8], ebx",
                "mov [{notes} + ecx + 12], edi",
                "inc dword ptr [{count}]",
                "pop ecx",
                "ret",
                count = sym NOTE_COUNT,
                notes = sym NOTES,
                id = const $id,
            )
        }
    };
}
noting_stand_in!(click_stand_in, 1);
noting_stand_in!(callback_stand_in, 2);
noting_stand_in!(screenshot_stand_in, 3);
noting_stand_in!(up_stand_in, 4);
noting_stand_in!(down_stand_in, 5);
noting_stand_in!(right_stand_in, 6);
noting_stand_in!(left_stand_in, 7);

/// Stands in for `input::take_key`: the test's key in al, the zero flag as the real one.
#[unsafe(naked)]
pub unsafe extern "C" fn take_key_stand_in() {
    core::arch::naked_asm!(
        "mov al, byte ptr [{key}]",
        "test al, al",
        "ret",
        key = sym TEST_KEY,
    )
}

fn note(id: u32, regs: &Regs) {
    let slot = (NOTE_COUNT.fetch_add(1, Ordering::SeqCst) as usize & (NOTE_SLOTS - 1)) * 4;
    for (n, value) in [id, regs.eax, regs.ebx, regs.edi].into_iter().enumerate() {
        NOTES[slot + n].store(value, Ordering::SeqCst);
    }
}

pub fn clear_notes() {
    for n in NOTES.iter().chain([&NOTE_COUNT]) {
        n.store(0, Ordering::SeqCst);
    }
}

pub fn notes() -> Vec<u32> {
    NOTES
        .iter()
        .chain([&NOTE_COUNT])
        .map(|n| n.load(Ordering::SeqCst))
        .collect()
}

/// Notes what the stand-ins note.
pub struct Noting;

impl Outside for Noting {
    fn click(&mut self, regs: &Regs) {
        note(CLICK, regs);
    }
    fn callback(&mut self, _: u32, regs: &Regs) {
        note(CALLBACK, regs);
    }
    fn screenshot(&mut self, regs: &Regs) {
        note(SCREENSHOT, regs);
    }
    unsafe fn move_highlight(&mut self, direction: Direction, regs: &Regs) -> Result<(), BadInput> {
        note(MOVE + direction as u32, regs);
        Ok(())
    }
}

/// The self-test of one of the four moves: a copy of the original whose click goes to a
/// stand-in, on a menu of five items whose callbacks are stand-ins too.
pub fn check_move(
    t: &mut SelfTest,
    direction: Direction,
    original: usize,
    len: usize,
    entry_bytes: &[u8],
    call_offsets: &[usize],
) -> Result<(), &'static str> {
    let redirect =
        |target: usize| (target == CLICK_RVA).then_some(click_stand_in as *const () as usize);
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(original),
            len,
            entry_bytes,
            call_offsets,
            &redirect,
        )
    }?;

    // Item 1 has no neighbour, item 2 asks its callback, 3 leads to 4, which can be chosen
    // and leads to itself, and 5 leads to 3, which cannot be chosen.
    let links = [NO_LINK, ASK_ITEM, 4, 4, 3];
    let mut menu = vec![0xAAAA_AAAAu32; 1 + 5 * 0x5C / 4 + 1];
    for (n, link) in links.into_iter().enumerate() {
        let item = &mut menu[1 + n * 0x5C / 4..][..0x5C / 4];
        for word in &mut item[0x24 / 4..0x34 / 4] {
            *word = link;
        }
        for word in &mut item[0x34 / 4..0x44 / 4] {
            *word = callback_stand_in as *const () as u32;
        }
        item[0x48 / 4] = if n == 3 { CAN_BE_CHOSEN } else { 0 };
    }
    *menu.last_mut().unwrap() = u32::MAX;

    let regions = move_regions();
    let inputs = [
        at(CURRENT_MENU_RVA),
        at(TURNING_RVA),
        at(TURN_COUNT_RVA),
        at(QUIET_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    unsafe { game_write(CURRENT_MENU_RVA, menu.as_ptr() as u32) };

    for highlighted in [1u32, 2, 3, 4, 5] {
        for turning in [0u32, 1] {
            for (count, turn) in [(1u32, 0u32), (3, 0), (3, 2)] {
                for way in [1u32, u32::MAX, 0] {
                    for quiet in [0u32, 1] {
                        let prepare = || unsafe {
                            game_write(MENU_ITEM_RVA, highlighted);
                            game_write(TURNING_RVA, turning);
                            game_write(TURN_COUNT_RVA, count);
                            game_write(TURN_RVA, turn);
                            game_write(TURN_WAY_RVA, way);
                            game_write(TURN_STEP_RVA, 0x5EEDu32);
                            game_write(QUIET_RVA, quiet);
                            clear_notes();
                        };
                        let before = Regs::SENTINEL;
                        prepare();
                        let mut ro = before;
                        unsafe { call_game(&mut ro, copy.addr()) };
                        let theirs = (capture(&regions), notes());
                        prepare();
                        let outcome = unsafe { move_highlight(direction, &before, &mut Noting) };
                        let mine = (capture(&regions), notes());
                        let preserved = ro.preserved_except(&before, 0);
                        t.check(outcome.is_ok() && theirs == mine && preserved, || {
                            format!(
                                "highlighted={highlighted} turning={turning} count={count} turn={turn} way={way:#x} quiet={quiet}: original {theirs:x?} preserved={preserved}; rust {outcome:?} {mine:x?}"
                            )
                        });
                    }
                }
            }
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    Ok(())
}
