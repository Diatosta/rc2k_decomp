//! The loop every menu screen runs, from entering the screen to leaving it. The game has 83
//! copies of it, one per screen, that differ in what `Screen` describes; each is hooked from
//! a file of its own that holds its `Screen`, and this is what they share.
//!
//! Once, on entry: the screen's `before` steps.
//!
//! Set-up (again after every action that does not leave the screen): the `set_up_first`
//! steps; `dword_758806` = 0; the highlight comes back from the screen's own variable; the
//! `prepare` steps; the menu and the function lists are put in place (a list marked `run`
//! is run once, here); `dword_7556A4` = 0x6F and `dword_755561` = 0; the `background` steps;
//! one frame is drawn (`menu::draw_frame`, `menu::update_blink`, then `screens::run_frame`
//! with the screen's frame callback, before which the lists are put in place again: the
//! `first_frame_lists` if the screen has any), with the time noted in `dword_5189A0`; the
//! `after_first_frame` steps.
//!
//! Every frame: `menu::update_blink`; leave if the game is quitting; `screens::run_frame`;
//! `dword_613830` = milliseconds since set-up; then `menu::handle_keys`:
//!   1  next frame
//!   2  (Escape) leave
//!   3  (Enter) the highlighted item's action runs (`menu::run_item_action`, with the
//!      highlight saved to the screen's variable before and taken back from it after).
//!      Then: leave if `dword_755569` or `dword_755980` is 1, or if `dword_755561` is 1
//!      (which is cleared); else next frame if `dword_755A38` is 2; else leave if
//!      `dword_75555D` is 1 (clearing `dword_755561`); else set up again.
//!   else leave
//! The same action check is made at the top of a frame whose `dword_7556A4` is 0x14D.
//! Three things a screen may have besides (`escape_leaves`, `each_frame`,
//! `short_action_check`) are described at those fields of `Screen`.
//!
//! Leaving: nothing more happens if the screen's `leave_at_once` variable is 1. On a screen
//! that asks (`confirm`), if `dword_758806` is 1,
//! `menu::confirm_leaving` puts the question; unless the answer (`g_yesNoAnswer`) is 1, the
//! `on_refusal` steps run and the screen is set up again; after a "yes" the `confirmed`
//! steps run. Otherwise the highlight is saved and the `after` steps run. Two variants: the
//! question is put as well when the `ask_also_if` variable is 1; and with
//! `unasked_skips_save`, a screen left without the question only runs its `after` steps.
//! More of the same kind are at their fields of `Screen`: `before_asking`, `ask_unless`,
//! `leaving`, `leave_saved_item`, `keys_menu`, `after_restored` and `result` (two loops
//! return a value in eax).
//!
//! Wherever the highlight is saved or taken back, `dword_755A3C` and `dword_755561` are
//! cleared if `dword_7558D8` and `dword_755A3C` are both 1.
//!
//! The registers are carried through as the original has them, because the functions it
//! calls get them: `handle_keys` reads ah on Enter, which here is a byte of the elapsed time.
//!
//! What the loop calls goes through `Env`, by the function's address in the game: the real
//! one runs the Rust version of what is ported and the game's code for the rest; the
//! self-test's follows a script. The game is inside a screen loop for as long as the screen
//! is up, so these hooks have ungated thunks and run the original alone in shadow mode, and
//! each real call writes a line to hook.log saying which version ran.

use crate::errors::FatalError;
use crate::framework::{Regs, game_addr, game_read, game_write, log, rva};
use crate::input::take_key::take_key;
use crate::menu::handle_keys::handle_keys;
use crate::menu::handle_yes_no::LEAVING_RVA;
use crate::menu::item_at::CURRENT_MENU_RVA;
use crate::menu::navigation::Game;
use crate::menu::run_item_action::run_item_action;
use crate::menu::run_list::run_list;
use crate::menu::table_calls::call as table_call;
use crate::menu::update_blink::update_blink;
use crate::menu::update_frame_clock::CLOCK_RVA;
use crate::service::state::MENU_ITEM_RVA;

// ---- game code the loop calls itself ----

pub const TAKE_KEY: usize = rva(0x407F95); // input::take_key
pub const RUN_LIST: usize = rva(0x468202); // menu::run_list
pub const DRAW_FRAME: usize = rva(0x47C420); // menu::draw_frame
pub const UPDATE_BLINK: usize = rva(0x47E826); // menu::update_blink
pub const RUN_FRAME: usize = rva(0x405A71); // screens::run_frame
pub const RUN_ITEM_ACTION: usize = rva(0x474683); // menu::run_item_action
pub const HANDLE_KEYS: usize = rva(0x465FCC); // menu::handle_keys
pub const CONFIRM_LEAVING: usize = rva(0x46FA90); // menu::confirm_leaving
pub const FRAME_CALLBACK: usize = rva(0x474D4B); // menu::frame_callback
const NULLSUB_83: usize = rva(0x471391); // nullsub_83: a lone `retn`
const NULLSUB_84: usize = rva(0x4713DD); // nullsub_84: a lone `retn`

// ---- game data ----

const ESCAPE_KEPT_RVA: usize = rva(0x7558D8); // g_clearsEscapeFlag
const ESCAPED_RVA: usize = rva(0x755A3C); // dword_755A3C
const BACK_RVA: usize = rva(0x755561); // dword_755561
const WENT_BACK_RVA: usize = rva(0x75555D); // dword_75555D
const SOUND_RVA: usize = rva(0x7556A4); // dword_7556A4: 0x14D = Enter was pressed
const DONE_A_RVA: usize = rva(0x755569); // dword_755569
const DONE_B_RVA: usize = rva(0x755980); // dword_755980
const ACTION_VALUE_RVA: usize = rva(0x755A38); // dword_755A38: 2 = stay on the screen as it is
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer
const KEYS_MENU_RVA: usize = rva(0x731BBB); // dword_731BBB
const SETUP_TIME_RVA: usize = rva(0x5189A0); // dword_5189A0
const ELAPSED_RVA: usize = rva(0x613830); // dword_613830
const QUIT_RVA: usize = rva(0x51837C); // dword_51837C

const NO_SOUND: u32 = 0x6F;
const ENTER_SOUND: u32 = 0x14D;

// ---- what differs from screen to screen ----

/// Something a screen does at one of the loop's fixed places.
#[derive(Clone, Copy, Debug)]
pub enum Step {
    /// Calls the game function at this RVA.
    Call(usize),
    /// Writes a game variable.
    Write(usize, u32),
    WriteF32(usize, f32),
    /// Loads eax.
    Eax(u32),
    /// Loads eax with the address of something in the game (a path, say).
    EaxAddr(usize),
    /// Calls the game function at this RVA, and if it comes back with the carry flag set,
    /// that is the fatal error with this code (and no message).
    CallOrFatal(usize, u32),
    /// Something only this screen does, written out in its file. What it calls in the game
    /// goes through the `Env`, like the loop's own calls.
    Do(unsafe fn(&mut Regs, &mut dyn Env)),
    /// The same for a step that can also send the loop somewhere: a name editor called once
    /// a frame that keeps the keys from the menu while a name is typed. Only looked at in
    /// `each_frame`.
    DoFlow(unsafe fn(&mut Regs, &mut dyn Env) -> Flow),
}

/// Where a `Step::DoFlow` sends the loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    /// Nowhere: the loop goes on with what comes next.
    On,
    /// To the top of the loop, without the action check and the keys.
    NextFrame,
    Leave,
    SetUp,
}

/// One of the screen's function lists: the table, and the variable that points at it.
#[derive(Clone, Copy, Debug)]
pub struct List {
    pub pointer: usize,
    pub table: usize,
    /// Run with `menu::run_list` at set-up, before its pointer is stored. Such a list is not
    /// put in place again before each frame; the others are.
    pub run: bool,
}

pub struct Screen {
    /// off_...: the screen's menu.
    pub menu: usize,
    /// The variable the highlight is kept in between visits.
    pub saved_item: usize,
    pub lists: &'static [List],
    /// The lists put in place before the first frame after set-up, where they are not the
    /// same as before every other frame: 35 loops put their 3D list in `g_unreadList` at
    /// set-up and before each frame, and a list of their own in `g_sceneList` only here.
    pub first_frame_lists: Option<&'static [List]>,
    pub frame_callback: usize,
    /// The frame callback of every frame but the first after set-up, where it is another
    /// one: the inspection area's loop is the only one.
    pub later_frame_callback: Option<usize>,
    pub before: &'static [Step],
    pub set_up_first: &'static [Step],
    pub prepare: &'static [Step],
    /// Steps between putting the lists in place and the background.
    pub after_lists: &'static [Step],
    pub background: &'static [Step],
    pub after_first_frame: &'static [Step],
    /// Some if leaving is asked about: what runs after a "no".
    pub confirm: Option<&'static [Step]>,
    /// A variable that, when it is 1, has the question put as well (`dword_755569` in the
    /// menus after the results screens).
    pub ask_also_if: Option<usize>,
    /// What runs after a "yes".
    pub confirmed: &'static [Step],
    /// When the question is not put, the highlight is not saved and the Escape flag is left
    /// alone: only the `after` steps run.
    pub unasked_skips_save: bool,
    pub after: &'static [Step],
    /// The Escape flag `dword_755A3C` is looked at at the top of every frame, before the quit
    /// switch, and the screen is left when it is 1.
    pub escape_leaves: bool,
    /// Steps of the screen's own once a frame, after the frame and before the keys.
    pub each_frame: &'static [Step],
    /// After an action only `dword_755569` and `dword_755561` are looked at; unless the
    /// screen is left it is set up again, whatever `dword_755A38` and `dword_75555D` hold.
    pub short_action_check: bool,
    /// A variable that, when it is 1 at the moment of leaving, makes the loop return at once:
    /// no question, the highlight not saved, no `after` steps.
    pub leave_at_once: Option<usize>,
    /// "Next frame" (after `handle_keys` returns 1, and after an action that leaves 2 in
    /// `dword_755A38`) goes to the `after_first_frame` steps, not past them. A `Flow` those
    /// steps return is honoured on every screen.
    pub next_frame_repeats_first: bool,
    /// The variable the highlight is kept in around an item's action, where it is not
    /// `saved_item`.
    pub action_saved_item: Option<usize>,
    /// The menu whose item's action is run, where it is not `menu`: three loops name a
    /// neighbouring screen's menu there (the original's slip, reproduced).
    pub action_menu: Option<usize>,
    /// The quit switch is not looked at; `dword_7556A4` = 0x6F is stored once a frame in
    /// that place.
    pub no_quit_check: bool,
    /// On a screen that asks: what runs before the question is put.
    pub before_asking: &'static [Step],
    /// A variable that, when it is 1, keeps the question from being put although
    /// `dword_758806` is 1.
    pub ask_unless: Option<usize>,
    /// On a screen that does not ask: what runs first on leaving, before the highlight is
    /// saved (the mode menu calls `menu::go_back` there when `dword_758806` is 1).
    pub leaving: &'static [Step],
    /// The variable the highlight is saved in on leaving, where it is not `saved_item`.
    pub leave_saved_item: Option<usize>,
    /// The menu stored in `dword_731BBB` before `menu::handle_keys`, where it is not `menu`.
    pub keys_menu: Option<usize>,
    /// What runs last of all, with the registers back as they were at entry.
    pub after_restored: &'static [Step],
    /// What the loop returns in eax, where it returns something: read from the game's
    /// variables as the loop leaves them.
    pub result: Option<unsafe fn() -> u32>,
    /// `dword_758806` is cleared and the highlight put in place (with the Escape check) after
    /// the first frame, not at the start of set-up, and set-up stores neither `dword_7556A4`
    /// nor `dword_755561` after the lists: the main menu.
    pub late_highlight: bool,
    /// Steps before the first frame after set-up, with the clock in eax.
    pub before_first_frame: &'static [Step],
    /// Steps before every other frame.
    pub before_frame: &'static [Step],
    /// Steps right after `update_blink` at the top of a frame; a `Flow` is honoured.
    pub after_blink: &'static [Step],
    /// After an action: left when `dword_755569` is 1; when `dword_755980` is 1 the answer
    /// `g_yesNoAnswer` is cleared, a key taken and the screen set up again; left when
    /// `dword_755561` is 1, which stays 1; otherwise `dword_7556A4` = 0x6F and set up again.
    pub restart_action_check: bool,
    /// The question is put whenever the screen is left, and after a "yes" the loop returns
    /// at once: the highlight is not saved.
    pub always_asks: bool,
}

/// The functions the loop calls, by RVA, and the quit switch it reads.
pub trait Env {
    fn call(&mut self, function: usize, regs: &mut Regs);
    fn quitting(&self) -> bool;
    /// A line for hook.log from a screen's own step; a self-test writes nothing.
    fn note(&mut self, _text: &str) {}
}

/// The game: the Rust versions of what is ported, the game's code for the rest.
pub struct Real;

impl Env for Real {
    fn call(&mut self, function: usize, regs: &mut Regs) {
        let entry = *regs;
        // A ported function that refuses its input goes to its hook, which deals with it.
        let done = unsafe {
            match function {
                TAKE_KEY => {
                    regs.eax = regs.eax & !0xFF | take_key() as u32;
                    true
                }
                UPDATE_BLINK => {
                    update_blink();
                    true
                }
                RUN_LIST => run_list(regs.esi, &entry, &mut table_call).is_ok(),
                RUN_ITEM_ACTION => run_item_action(regs.edi, &entry, &mut table_call).is_ok(),
                // `nullsub_83`, `nullsub_84` (the review options' set-up and leaving).
                NULLSUB_83 | NULLSUB_84 => true,
                HANDLE_KEYS => {
                    crate::fatal::raise::raise_test_if_armed(regs);
                    handle_keys(&entry, &mut Game)
                        .map(|left| left.apply(regs))
                        .is_ok()
                }
                f => crate::menu::set_up_calls::call(f, regs),
            }
        };
        if !done {
            crate::menu::table_calls::call_any(game_addr(function) as u32, regs);
        }
    }

    fn quitting(&self) -> bool {
        unsafe { game_read::<u32>(QUIT_RVA) == 1 }
    }

    fn note(&mut self, text: &str) {
        log(text);
    }
}

// ---- Rust version ----

/// Where the loop goes next.
#[derive(Clone, Copy)]
enum Next {
    SetUp,
    FrameTop,
    Frame,
    ActionCheck,
    Leave,
}

/// Clears the Escape flags if they are kept from one screen to the next and are set.
unsafe fn clear_escape() {
    unsafe {
        if game_read::<u32>(ESCAPE_KEPT_RVA) == 1 && game_read::<u32>(ESCAPED_RVA) == 1 {
            game_write(ESCAPED_RVA, 0u32);
            game_write(BACK_RVA, 0u32);
        }
    }
}

/// A call after which a set carry flag is the fatal error `code`.
pub unsafe fn call_or_fatal(function: usize, code: u32, g: &mut Regs, env: &mut dyn Env) {
    env.call(function, g);
    if g.carry() {
        log(&format!(
            "a screen loop's Rust version raises fatal error {code:#x}"
        ));
        unsafe { crate::fatal::raise::raise(FatalError { code, message: "" }, g) };
    }
}

// What the generated `Step::Do` functions use for the instructions that have no step.

/// `pusha`: the registers, to be put back by `popa`.
pub fn pusha(g: &Regs, stack: &mut Vec<Regs>) {
    stack.push(*g);
}

/// `popa`: the registers as they were at the matching `pusha`; the flags stay.
pub fn popa(g: &mut Regs, stack: &mut Vec<Regs>) {
    let flags = g.eflags;
    *g = stack.pop().expect("a popa without its pusha");
    g.eflags = flags;
}

/// `rep movsb`: ecx bytes from esi to edi.
pub unsafe fn copy_bytes(g: &mut Regs) {
    unsafe {
        std::ptr::copy(g.esi as *const u8, g.edi as *mut u8, g.ecx as usize);
    }
    g.esi = g.esi.wrapping_add(g.ecx);
    g.edi = g.edi.wrapping_add(g.ecx);
    g.ecx = 0;
}

pub unsafe fn read_at(address: u32) -> u32 {
    unsafe { (address as *const u32).read_unaligned() }
}

/// Runs the steps of one place. Anything but `Flow::On` comes from a `DoFlow` step, after
/// which the rest of the place is not run.
unsafe fn run(steps: &[Step], g: &mut Regs, env: &mut dyn Env) -> Flow {
    for step in steps {
        unsafe {
            match *step {
                Step::DoFlow(what) => match what(g, env) {
                    Flow::On => {}
                    elsewhere => return elsewhere,
                },
                Step::Call(function) => env.call(function, g),
                Step::Write(variable, value) => game_write(variable, value),
                Step::WriteF32(variable, value) => game_write(variable, value),
                Step::Eax(value) => g.eax = value,
                Step::EaxAddr(what) => g.eax = game_addr(what) as u32,
                Step::CallOrFatal(function, code) => call_or_fatal(function, code, g, env),
                Step::Do(what) => what(g, env),
            }
        }
    }
    Flow::On
}

/// Puts the menu and the lists in place, as the original does before a frame.
unsafe fn set_lists(screen: &Screen, lists: &[List], g: &mut Regs) {
    unsafe {
        g.edi = game_addr(screen.menu) as u32;
        game_write(CURRENT_MENU_RVA, g.edi);
        for list in lists.iter().filter(|list| !list.run) {
            g.esi = game_addr(list.table) as u32;
            game_write(list.pointer, g.esi);
        }
    }
}

/// `entry` are the registers at entry.
/// Returns why the screen was left, for hook.log.
pub unsafe fn screen_loop(screen: &Screen, entry: &Regs, env: &mut dyn Env) -> String {
    let menu = game_addr(screen.menu) as u32;
    let callback = game_addr(screen.frame_callback) as u32;
    let later_callback =
        game_addr(screen.later_frame_callback.unwrap_or(screen.frame_callback)) as u32;
    let keys_menu = screen
        .keys_menu
        .map_or(menu, |other| game_addr(other) as u32);
    let mut g = *entry;
    let (mut frames, mut actions, mut why) = (0u32, 0u32, String::new());
    unsafe {
        run(screen.before, &mut g, env);

        let action_menu = screen
            .action_menu
            .map_or(menu, |other| game_addr(other) as u32);
        let next_frame = if screen.next_frame_repeats_first {
            Next::FrameTop
        } else {
            Next::Frame
        };
        let mut next = Next::SetUp;
        loop {
            next = match next {
                Next::SetUp => {
                    run(screen.set_up_first, &mut g, env);
                    if !screen.late_highlight {
                        game_write(LEAVING_RVA, 0u32);
                        g.edi = menu;
                        g.ebx = game_read(screen.saved_item);
                        game_write(MENU_ITEM_RVA, g.ebx);
                        clear_escape();
                    }
                    run(screen.prepare, &mut g, env);

                    g.edi = menu;
                    game_write(CURRENT_MENU_RVA, menu);
                    for list in screen.lists {
                        g.esi = game_addr(list.table) as u32;
                        if list.run {
                            env.call(RUN_LIST, &mut g);
                        }
                        game_write(list.pointer, g.esi);
                    }
                    run(screen.after_lists, &mut g, env);
                    if !screen.late_highlight {
                        game_write(SOUND_RVA, NO_SOUND);
                        game_write(BACK_RVA, 0u32);
                    }
                    run(screen.background, &mut g, env);
                    g.edi = menu;
                    env.call(DRAW_FRAME, &mut g);
                    env.call(UPDATE_BLINK, &mut g);
                    g.eax = game_read(CLOCK_RVA);
                    game_write(SETUP_TIME_RVA, g.eax);
                    game_write(ELAPSED_RVA, 0u32);
                    run(screen.before_first_frame, &mut g, env);
                    g.eax = callback;
                    set_lists(
                        screen,
                        screen.first_frame_lists.unwrap_or(screen.lists),
                        &mut g,
                    );
                    env.call(RUN_FRAME, &mut g);
                    if screen.late_highlight {
                        game_write(LEAVING_RVA, 0u32);
                        g.edi = menu;
                        g.ebx = game_read(screen.saved_item);
                        game_write(MENU_ITEM_RVA, g.ebx);
                        clear_escape();
                    }
                    Next::FrameTop
                }
                Next::FrameTop => match run(screen.after_first_frame, &mut g, env) {
                    Flow::On | Flow::NextFrame => Next::Frame,
                    Flow::SetUp => Next::SetUp,
                    Flow::Leave => {
                        why = "a step of the screen's own".into();
                        Next::Leave
                    }
                },
                Next::Frame => {
                    env.call(UPDATE_BLINK, &mut g);
                    frames += 1;
                    let flow = run(screen.after_blink, &mut g, env);
                    if flow == Flow::SetUp {
                        Next::SetUp
                    } else if flow == Flow::Leave {
                        why = "a step of the screen's own".into();
                        Next::Leave
                    } else if screen.escape_leaves && game_read::<u32>(ESCAPED_RVA) == 1 {
                        why = "the Escape flag was set".into();
                        Next::Leave
                    } else if !screen.no_quit_check && env.quitting() {
                        why = "the game is quitting".into();
                        Next::Leave
                    } else {
                        if screen.no_quit_check {
                            game_write(SOUND_RVA, NO_SOUND);
                        }
                        run(screen.before_frame, &mut g, env);
                        g.eax = later_callback;
                        set_lists(screen, screen.lists, &mut g);
                        env.call(RUN_FRAME, &mut g);
                        g.eax = game_read::<u32>(CLOCK_RVA).wrapping_sub(game_read(SETUP_TIME_RVA));
                        game_write(ELAPSED_RVA, g.eax);
                        match run(screen.each_frame, &mut g, env) {
                            Flow::On => Next::ActionCheck,
                            Flow::NextFrame => Next::Frame,
                            Flow::SetUp => Next::SetUp,
                            Flow::Leave => {
                                why = "a step of the screen's own".into();
                                Next::Leave
                            }
                        }
                    }
                }
                Next::ActionCheck if game_read::<u32>(SOUND_RVA) != ENTER_SOUND => {
                    game_write(KEYS_MENU_RVA, keys_menu);
                    env.call(HANDLE_KEYS, &mut g);
                    match g.eax {
                        1 => next_frame,
                        3 => Next::ActionCheck,
                        other => {
                            why = format!("handle_keys returned {other}");
                            Next::Leave
                        }
                    }
                }
                Next::ActionCheck => {
                    let elapsed = g.eax;
                    actions += 1;
                    g.ecx = game_read(MENU_ITEM_RVA);
                    let item = g.ecx;
                    let kept = screen.action_saved_item.unwrap_or(screen.saved_item);
                    game_write(kept, g.ecx);
                    clear_escape();
                    g.edi = action_menu;
                    env.call(RUN_ITEM_ACTION, &mut g);
                    game_write(LEAVING_RVA, 0u32);
                    g.edi = action_menu;
                    g.ebx = game_read(kept);
                    game_write(MENU_ITEM_RVA, g.ebx);
                    clear_escape();
                    g.eax = elapsed;
                    if screen.restart_action_check {
                        if game_read::<u32>(DONE_A_RVA) == 1 {
                            why = format!("after the action of item {item}: dword_755569 was 1");
                            Next::Leave
                        } else if game_read::<u32>(DONE_B_RVA) == 1 {
                            game_write(ANSWER_RVA, 0u32);
                            env.call(TAKE_KEY, &mut g);
                            Next::SetUp
                        } else if game_read::<u32>(BACK_RVA) == 1 {
                            why = format!("after the action of item {item}: dword_755561 is 1");
                            Next::Leave
                        } else {
                            game_write(SOUND_RVA, NO_SOUND);
                            Next::SetUp
                        }
                    } else if game_read::<u32>(DONE_A_RVA) == 1
                        || (!screen.short_action_check && game_read::<u32>(DONE_B_RVA) == 1)
                    {
                        why = format!(
                            "after the action of item {item}: dword_755569={} dword_755980={}",
                            game_read::<u32>(DONE_A_RVA),
                            game_read::<u32>(DONE_B_RVA)
                        );
                        Next::Leave
                    } else if game_read::<u32>(BACK_RVA) == 1 {
                        game_write(BACK_RVA, 0u32);
                        why = format!("after the action of item {item}: dword_755561 was 1");
                        Next::Leave
                    } else if screen.short_action_check {
                        Next::SetUp
                    } else {
                        game_write(SOUND_RVA, NO_SOUND);
                        if game_read::<u32>(ACTION_VALUE_RVA) == 2 {
                            next_frame
                        } else if game_read::<u32>(WENT_BACK_RVA) == 1 {
                            game_write(BACK_RVA, 0u32);
                            why = format!("after the action of item {item}: dword_75555D is 1");
                            Next::Leave
                        } else {
                            Next::SetUp
                        }
                    }
                }
                Next::Leave => {
                    if screen
                        .leave_at_once
                        .is_some_and(|flag| game_read::<u32>(flag) == 1)
                    {
                        return format!("{why}; {frames} frames, {actions} actions");
                    }
                    if let Some(on_refusal) = screen.confirm {
                        let asked = screen.always_asks
                            || (game_read::<u32>(LEAVING_RVA) == 1
                                && !screen
                                    .ask_unless
                                    .is_some_and(|flag| game_read::<u32>(flag) == 1))
                            || screen
                                .ask_also_if
                                .is_some_and(|flag| game_read::<u32>(flag) == 1);
                        if asked {
                            run(screen.before_asking, &mut g, env);
                            env.call(CONFIRM_LEAVING, &mut g);
                            if game_read::<u32>(ANSWER_RVA) != 1 {
                                run(on_refusal, &mut g, env);
                                next = Next::SetUp;
                                continue;
                            }
                            run(screen.confirmed, &mut g, env);
                            if screen.always_asks {
                                return format!("{why}; {frames} frames, {actions} actions");
                            }
                        } else if screen.unasked_skips_save {
                            run(screen.after, &mut g, env);
                            g = *entry;
                            run(screen.after_restored, &mut g, env);
                            return format!("{why}; {frames} frames, {actions} actions");
                        }
                    } else {
                        run(screen.leaving, &mut g, env);
                    }
                    g.ecx = game_read(MENU_ITEM_RVA);
                    game_write(screen.leave_saved_item.unwrap_or(screen.saved_item), g.ecx);
                    clear_escape();
                    run(screen.after, &mut g, env);
                    g = *entry;
                    run(screen.after_restored, &mut g, env);
                    return format!("{why}; {frames} frames, {actions} actions");
                }
            };
        }
    }
}

// ---- hook ----

/// `dispatch_screen` outside shadow mode, without its hook (typed tables, 2026-10-22: the entry the menus' and draw lists' enums run, with no hook).
pub fn screen_entry(
    name: &str,
    r: &mut Regs,
    screen: &Screen,
) -> Result<(), crate::errors::Refused> {
    log(&format!("{name}: the Rust version runs"));
    let entry = *r;
    let why = unsafe { screen_loop(screen, &entry, &mut Real) };
    if let Some(result) = screen.result {
        r.eax = unsafe { result() };
    }
    log(&format!(
        "{name}: the Rust version has left the screen ({why})"
    ));
    Ok(())
}

/// A screen loop run from Rust: its Rust version, as its hook runs it outside shadow mode
/// (eax the screen's result where it has one; every other register kept).
pub unsafe fn run_screen(screen: &Screen, r: &mut Regs) {
    unsafe { screen_loop(screen, r, &mut Real) };
    if let Some(result) = screen.result {
        r.eax = unsafe { result() };
    }
}
