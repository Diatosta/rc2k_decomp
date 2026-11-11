//! The loop every menu screen runs, from entering the screen to leaving it, as a sequence of
//! decisions over a host. The host does what the game does at each place (runs the screen's
//! steps, draws a frame, polls the keys, reads and writes the menus' variables); the loop
//! here decides the order and where to go next. The screen's description is in `Shape`.
//!
//! Once, on entry: the screen's `Before` steps.
//!
//! Set-up (again after every action that does not leave the screen): `SetUpFirst`; the
//! highlight comes back (unless the screen's is late); `Prepare`; the menu and the lists are
//! put in place; `AfterLists`; the menu sound and the leave switch are cleared (unless the
//! highlight is late); `Background`; one frame is drawn and the set-up time noted;
//! `BeforeFirstFrame`; the lists put in place again; the first frame run.
//!
//! Every frame: blink; `AfterBlink`; leave on the Escape flag or the quit switch; the debug
//! screens' turn; `BeforeFrame`; the lists; the frame; the time since set-up noted;
//! `EachFrame`; then the keys: next frame, leave (Escape) or the highlighted item's action,
//! after which the action check says where the loop goes.
//!
//! Leaving: nothing more happens if the screen leaves at once. A screen that asks puts the
//! question when it was left by a key (or always, or by its own second switch); a "no" runs
//! `OnRefusal` and sets the screen up again, a "yes" runs `Confirmed`. Then the highlight is
//! saved and `After` and `AfterRestored` run.

use crate::menu::keys::Pressed;

/// Where a screen's steps are run from, in the order the loop reaches them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Before,
    SetUpFirst,
    Prepare,
    AfterLists,
    Background,
    BeforeFirstFrame,
    AfterFirstFrame,
    AfterBlink,
    BeforeFrame,
    EachFrame,
    /// Before the leaving question is put.
    AskBefore,
    OnRefusal,
    Confirmed,
    /// On a screen that does not ask: first on leaving.
    Leaving,
    After,
    AfterRestored,
}

/// Where a step that can send the loop somewhere sends it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    /// Nowhere: the loop goes on with what comes next.
    On,
    /// To the top of the loop, without the action check and the keys.
    NextFrame,
    Leave,
    SetUp,
}

/// What the highlighted item's action left for the loop to decide on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActionResult {
    /// The action asks to leave the screen (and, with its question, to be asked about it).
    pub ask_leave: bool,
    /// The event the screen belongs to is over.
    pub event_over: bool,
    /// The action leaves the screen.
    pub leave_screen: bool,
    /// The event flow behind the screen was left.
    pub event_left: bool,
    /// The item's value, which the loop tells a plain action (2) from others by.
    pub action_mode: u32,
}

/// What the loop looks at after a key opened the highlighted item's action.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionCheck {
    /// Leave if `ask_leave`, `event_over` or `leave_screen` is set (the last is cleared); else
    /// next frame if the action mode is 2; else leave if `event_left` is set; else set up
    /// again.
    Standard,
    /// Only `ask_leave` and `leave_screen` are looked at; unless the screen is left it is set
    /// up again, whatever the action mode and `event_left` hold.
    Short,
    /// Left when `ask_leave` is set; when `event_over` is set the answer is cleared, a key
    /// taken and the screen set up again; left when `leave_screen` is set, which stays 1;
    /// otherwise the menu sound is cleared and the screen set up again.
    Restart,
}

/// How a screen asks about being left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AskShape {
    /// The question is put whenever the screen is left, and after a "yes" the loop returns
    /// at once: the highlight is not saved.
    pub always: bool,
    /// When the question is not put, the highlight is not saved: only `After` and
    /// `AfterRestored` run.
    pub unasked_skips_save: bool,
}

/// What the loop needs to know about a screen (the rest the host knows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    /// The highlight comes back after the first frame, and set-up stores neither the menu
    /// sound nor the leave switch after the lists.
    pub late_highlight: bool,
    /// The Escape flag is looked at at the top of every frame, and leaves the screen.
    pub escape_leaves: bool,
    pub action_check: ActionCheck,
    /// The quit switch is not looked at; the menu sound is cleared once a frame instead.
    pub no_quit_check: bool,
    /// "Next frame" goes to the `AfterFirstFrame` steps, not past them.
    pub next_frame_repeats_first: bool,
    /// Some if leaving is asked about.
    pub asking: Option<AskShape>,
}

impl Shape {
    /// The shape of a plain screen.
    pub const DEFAULT: Shape = Shape {
        late_highlight: false,
        escape_leaves: false,
        action_check: ActionCheck::Standard,
        no_quit_check: false,
        next_frame_repeats_first: false,
        asking: None,
    };
}

/// What the game does at each place of the loop. Methods that can fail or run a screen's
/// steps return the host's error.
pub trait ScreenHost {
    type Error;

    /// Runs the screen's steps of this place; what a step that sends the loop somewhere says.
    fn run(&mut self, place: Place) -> Result<Flow, Self::Error>;

    // ---- set-up ----
    /// Clears "left by a key" and takes the highlight back from the screen's variable, with
    /// the Escape check.
    fn restore_highlight(&mut self);
    /// Puts the menu in place, and the lists (running those marked to run once).
    fn put_menu_and_lists(&mut self) -> Result<(), Self::Error>;
    fn clear_menu_sound(&mut self);
    fn clear_leave_screen(&mut self);
    /// Draws the frame the set-up shows before the first real one, and runs the blink.
    fn draw_set_up_frame(&mut self) -> Result<(), Self::Error>;
    /// Notes the set-up time and restarts the screen's clock.
    fn note_set_up_time(&mut self);
    /// Puts the lists in place for a frame: the first frame's, or the ones of every other.
    fn put_frame_lists(&mut self, first: bool);
    /// Runs the screen's frame callback (`first`: the one of the first frame).
    fn run_frame(&mut self, first: bool) -> Result<(), Self::Error>;

    // ---- a frame ----
    fn update_blink(&mut self);
    fn escape_flag(&self) -> bool;
    fn quitting(&self) -> bool;
    /// The window was closed: ends the game at once.
    fn leave_on_close(&mut self) -> !;
    /// The debug screens' turn; whether they want the screen set up again.
    fn debug_serve(&mut self) -> Result<bool, Self::Error>;
    /// Notes the time since set-up.
    fn note_frame_time(&mut self);

    // ---- keys and actions ----
    /// Hands the keys to the menu; what was pressed.
    fn handle_keys(&mut self) -> Result<Pressed, Self::Error>;
    /// Whether Enter left an action pending (the menu sound).
    fn action_pending(&self) -> bool;
    /// The highlighted item.
    fn highlight(&self) -> u32;
    /// Keeps the highlight in the variable around the action.
    fn keep_action_item(&mut self, item: u32);
    /// Clears the Escape flags if they are set; whether they were.
    fn clear_escape(&mut self) -> bool;
    fn run_item_action(&mut self) -> Result<ActionResult, Self::Error>;
    fn clear_left_by_key(&mut self);
    /// Takes the highlight back from the action's variable.
    fn restore_action_item(&mut self);
    fn clear_answer(&mut self);
    fn take_key(&mut self) -> Result<(), Self::Error>;

    // ---- leaving ----
    fn leave_at_once(&self) -> bool;
    fn left_by_key(&self) -> bool;
    /// The screen's variable that keeps the question from being put although it was left by
    /// a key is 1.
    fn ask_suppressed(&self) -> bool;
    /// The screen's second switch that has the question put as well is set.
    fn ask_also(&self) -> bool;
    fn confirm_leaving(&mut self) -> Result<(), Self::Error>;
    fn answered_yes(&self) -> bool;
    /// Saves the highlight in the variable it is kept in on leaving.
    fn save_leave_item(&mut self);
}

/// Why an action's result sent the loop out of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionExit {
    /// `ask_leave` set (and `event_over`, on a standard check).
    Asked { ask_leave: bool, event_over: bool },
    /// `ask_leave` set on a restarting check.
    AskedRestart,
    /// `leave_screen` set (cleared by a standard check, kept by a restarting one).
    LeaveScreen { cleared: bool },
    /// `event_left` set after an action that stays by default.
    EventLeft,
}

/// Why a screen was left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenExit {
    /// Escape was taken from the keys.
    Escape,
    /// The Escape flag was set at the top of a frame.
    EscapeFlag,
    /// A step of the screen's own sent the loop out.
    OwnStep,
    /// After the action of the highlighted item.
    Action { item: u32, how: ActionExit },
}

impl std::fmt::Display for ScreenExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ScreenExit::Escape => write!(f, "Escape"),
            ScreenExit::EscapeFlag => write!(f, "the Escape flag was set"),
            ScreenExit::OwnStep => write!(f, "a step of the screen's own"),
            ScreenExit::Action { item, how } => {
                write!(f, "after the action of item {item}: ")?;
                match how {
                    ActionExit::Asked {
                        ask_leave,
                        event_over,
                    } => write!(f, "ask_leave={ask_leave} event_over={event_over}"),
                    ActionExit::AskedRestart => write!(f, "ask_leave was 1"),
                    ActionExit::LeaveScreen { cleared: true } => write!(f, "leave_screen was 1"),
                    ActionExit::LeaveScreen { cleared: false } => write!(f, "leave_screen is 1"),
                    ActionExit::EventLeft => write!(f, "event_left is 1"),
                }
            }
        }
    }
}

/// How a screen loop ended: why, and how long it ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exit {
    pub reason: ScreenExit,
    pub frames: u32,
    pub actions: u32,
}

impl std::fmt::Display for Exit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}; {} frames, {} actions",
            self.reason, self.frames, self.actions
        )
    }
}

/// Where the loop goes next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Next {
    SetUp,
    FrameTop,
    Frame,
    ActionCheck,
    Keys,
    Leave,
}

/// How leaving ended.
enum Left {
    Done,
    SetUp,
}

/// The loop of one screen over its host.
struct Loop<'a, H: ScreenHost> {
    shape: &'a Shape,
    host: &'a mut H,
    /// Where the loop goes after a frame without a key.
    next_frame: Next,
    frames: u32,
    actions: u32,
    why: ScreenExit,
}

/// Runs a screen from entry to leaving. Returns why it was left.
pub fn screen_loop<H: ScreenHost>(shape: &Shape, host: &mut H) -> Result<Exit, H::Error> {
    host.run(Place::Before)?;
    let mut state = Loop {
        shape,
        host,
        next_frame: if shape.next_frame_repeats_first {
            Next::FrameTop
        } else {
            Next::Frame
        },
        frames: 0,
        actions: 0,
        why: ScreenExit::Escape,
    };
    let mut next = Next::SetUp;
    loop {
        next = match next {
            Next::SetUp => state.set_up()?,
            Next::FrameTop => state.after_first_frame()?,
            Next::Frame => state.frame()?,
            Next::ActionCheck if !state.host.action_pending() => Next::Keys,
            Next::ActionCheck => state.action()?,
            Next::Keys => state.keys()?,
            Next::Leave => match state.leave()? {
                Left::Done => {
                    return Ok(Exit {
                        reason: state.why,
                        frames: state.frames,
                        actions: state.actions,
                    });
                }
                Left::SetUp => Next::SetUp,
            },
        };
    }
}

impl<H: ScreenHost> Loop<'_, H> {
    /// Where a step's flow sends the loop, when it sends it anywhere.
    fn sent(&mut self, flow: Flow) -> Option<Next> {
        match flow {
            Flow::On => None,
            Flow::NextFrame => Some(Next::Frame),
            Flow::SetUp => Some(Next::SetUp),
            Flow::Leave => {
                self.why = ScreenExit::OwnStep;
                Some(Next::Leave)
            }
        }
    }

    /// Set-up, again after every action that does not leave the screen.
    fn set_up(&mut self) -> Result<Next, H::Error> {
        let late = self.shape.late_highlight;
        let host = &mut *self.host;
        host.run(Place::SetUpFirst)?;
        if !late {
            host.restore_highlight();
        }
        host.run(Place::Prepare)?;
        host.put_menu_and_lists()?;
        host.run(Place::AfterLists)?;
        if !late {
            host.clear_menu_sound();
            host.clear_leave_screen();
        }
        host.run(Place::Background)?;
        host.draw_set_up_frame()?;
        host.note_set_up_time();
        host.run(Place::BeforeFirstFrame)?;
        host.put_frame_lists(true);
        host.run_frame(true)?;
        if late {
            host.restore_highlight();
        }
        Ok(Next::FrameTop)
    }

    /// The `AfterFirstFrame` steps, once the first frame is drawn.
    fn after_first_frame(&mut self) -> Result<Next, H::Error> {
        let flow = self.host.run(Place::AfterFirstFrame)?;
        Ok(match flow {
            Flow::On | Flow::NextFrame => Next::Frame,
            elsewhere => self.sent(elsewhere).unwrap_or(Next::Frame),
        })
    }

    /// One frame: blink, quit and Escape checks, the frame and the per-frame steps.
    fn frame(&mut self) -> Result<Next, H::Error> {
        self.host.update_blink();
        self.frames += 1;
        let flow = self.host.run(Place::AfterBlink)?;
        if matches!(flow, Flow::SetUp | Flow::Leave) {
            return Ok(self.sent(flow).unwrap_or(Next::SetUp));
        }
        if self.shape.escape_leaves && self.host.escape_flag() {
            self.why = ScreenExit::EscapeFlag;
            return Ok(Next::Leave);
        }
        if !self.shape.no_quit_check && self.host.quitting() {
            self.host.leave_on_close();
        }
        if self.host.debug_serve()? {
            return Ok(Next::SetUp);
        }
        if self.shape.no_quit_check {
            self.host.clear_menu_sound();
        }
        self.host.run(Place::BeforeFrame)?;
        self.host.put_frame_lists(false);
        self.host.run_frame(false)?;
        self.host.note_frame_time();
        let flow = self.host.run(Place::EachFrame)?;
        Ok(match flow {
            Flow::On => Next::ActionCheck,
            elsewhere => self.sent(elsewhere).unwrap_or(Next::Frame),
        })
    }

    /// The keys of a frame and where they lead.
    fn keys(&mut self) -> Result<Next, H::Error> {
        Ok(match self.host.handle_keys()? {
            Pressed::Nothing => self.next_frame,
            Pressed::Enter => Next::ActionCheck,
            Pressed::Escape => {
                self.why = ScreenExit::Escape;
                Next::Leave
            }
        })
    }

    /// The highlighted item's action, and what it leaves: leave, next frame or set up again.
    fn action(&mut self) -> Result<Next, H::Error> {
        self.actions += 1;
        let host = &mut *self.host;
        let item = host.highlight();
        host.keep_action_item(item);
        host.clear_escape();
        let mut result = host.run_item_action()?;
        host.clear_left_by_key();
        host.restore_action_item();
        if host.clear_escape() {
            result.leave_screen = false;
        }
        match self.shape.action_check {
            ActionCheck::Restart => self.action_restart(item, result),
            check => Ok(self.action_checked(item, check == ActionCheck::Short, result)),
        }
    }

    /// The action check of `ActionCheck::Restart`.
    fn action_restart(&mut self, item: u32, result: ActionResult) -> Result<Next, H::Error> {
        let leave = |how| ScreenExit::Action { item, how };
        Ok(if result.ask_leave {
            self.why = leave(ActionExit::AskedRestart);
            Next::Leave
        } else if result.event_over {
            self.host.clear_answer();
            self.host.take_key()?;
            Next::SetUp
        } else if result.leave_screen {
            self.why = leave(ActionExit::LeaveScreen { cleared: false });
            Next::Leave
        } else {
            self.host.clear_menu_sound();
            Next::SetUp
        })
    }

    /// The action check of `ActionCheck::Standard` (`short`: of `ActionCheck::Short`).
    fn action_checked(&mut self, item: u32, short: bool, result: ActionResult) -> Next {
        let ask_leave = result.ask_leave;
        let event_over = result.event_over;
        let leave = |how| ScreenExit::Action { item, how };
        if ask_leave || (!short && event_over) {
            self.why = leave(ActionExit::Asked {
                ask_leave,
                event_over,
            });
            Next::Leave
        } else if result.leave_screen {
            self.host.clear_leave_screen();
            self.why = leave(ActionExit::LeaveScreen { cleared: true });
            Next::Leave
        } else if short {
            Next::SetUp
        } else {
            self.host.clear_menu_sound();
            if result.action_mode == 2 {
                self.next_frame
            } else if result.event_left {
                self.host.clear_leave_screen();
                self.why = leave(ActionExit::EventLeft);
                Next::Leave
            } else {
                Next::SetUp
            }
        }
    }

    /// Leaving: the question when the screen asks, then the `After` steps.
    fn leave(&mut self) -> Result<Left, H::Error> {
        let host = &mut *self.host;
        if host.leave_at_once() {
            return Ok(Left::Done);
        }
        if let Some(asking) = self.shape.asking {
            let asked =
                asking.always || (host.left_by_key() && !host.ask_suppressed()) || host.ask_also();
            if asked {
                host.run(Place::AskBefore)?;
                host.confirm_leaving()?;
                if !host.answered_yes() {
                    host.run(Place::OnRefusal)?;
                    return Ok(Left::SetUp);
                }
                host.run(Place::Confirmed)?;
                if asking.always {
                    return Ok(Left::Done);
                }
            } else if asking.unasked_skips_save {
                host.run(Place::After)?;
                host.run(Place::AfterRestored)?;
                return Ok(Left::Done);
            }
        } else {
            host.run(Place::Leaving)?;
        }
        host.save_leave_item();
        host.clear_escape();
        host.run(Place::After)?;
        host.run(Place::AfterRestored)?;
        Ok(Left::Done)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scripted host: records every effect, answers the keys from a script.
    struct Mock {
        log: Vec<String>,
        keys: Vec<Pressed>,
        pending: bool,
        results: Vec<ActionResult>,
        flows: Vec<(Place, Flow)>,
        left_by_key: bool,
        answers: Vec<bool>,
        answer: bool,
        leave_at_once: bool,
    }

    impl Mock {
        fn new(keys: Vec<Pressed>) -> Mock {
            Mock {
                log: Vec::new(),
                keys,
                pending: false,
                results: Vec::new(),
                flows: Vec::new(),
                left_by_key: false,
                answers: Vec::new(),
                answer: false,
                leave_at_once: false,
            }
        }
        fn note(&mut self, what: &str) {
            self.log.push(what.to_string());
        }
    }

    impl ScreenHost for Mock {
        type Error = ();
        fn run(&mut self, place: Place) -> Result<Flow, ()> {
            self.log.push(format!("{place:?}"));
            if let Some(at) = self.flows.iter().position(|(p, _)| *p == place) {
                return Ok(self.flows.remove(at).1);
            }
            Ok(Flow::On)
        }
        fn restore_highlight(&mut self) {
            self.note("restore_highlight");
        }
        fn put_menu_and_lists(&mut self) -> Result<(), ()> {
            self.note("lists");
            Ok(())
        }
        fn clear_menu_sound(&mut self) {
            self.pending = false;
            self.note("clear_sound");
        }
        fn clear_leave_screen(&mut self) {
            self.note("clear_leave");
        }
        fn draw_set_up_frame(&mut self) -> Result<(), ()> {
            self.note("draw");
            Ok(())
        }
        fn note_set_up_time(&mut self) {
            self.note("setup_time");
        }
        fn put_frame_lists(&mut self, first: bool) {
            self.note(if first {
                "frame_lists_first"
            } else {
                "frame_lists"
            });
        }
        fn run_frame(&mut self, first: bool) -> Result<(), ()> {
            self.note(if first { "frame_first" } else { "frame" });
            Ok(())
        }
        fn update_blink(&mut self) {
            self.note("blink");
        }
        fn escape_flag(&self) -> bool {
            false
        }
        fn quitting(&self) -> bool {
            false
        }
        fn leave_on_close(&mut self) -> ! {
            panic!("closed");
        }
        fn debug_serve(&mut self) -> Result<bool, ()> {
            Ok(false)
        }
        fn note_frame_time(&mut self) {
            self.note("frame_time");
        }
        fn handle_keys(&mut self) -> Result<Pressed, ()> {
            self.note("keys");
            let key = self.keys.remove(0);
            if key == Pressed::Enter {
                self.pending = true;
            }
            Ok(key)
        }
        fn action_pending(&self) -> bool {
            self.pending
        }
        fn highlight(&self) -> u32 {
            3
        }
        fn keep_action_item(&mut self, item: u32) {
            self.log.push(format!("keep {item}"));
        }
        fn clear_escape(&mut self) -> bool {
            self.note("clear_escape");
            false
        }
        fn run_item_action(&mut self) -> Result<ActionResult, ()> {
            self.note("action");
            Ok(self.results.remove(0))
        }
        fn clear_left_by_key(&mut self) {
            self.note("clear_left");
        }
        fn restore_action_item(&mut self) {
            self.note("restore_action_item");
        }
        fn clear_answer(&mut self) {
            self.note("clear_answer");
        }
        fn take_key(&mut self) -> Result<(), ()> {
            self.note("take_key");
            Ok(())
        }
        fn leave_at_once(&self) -> bool {
            self.leave_at_once
        }
        fn left_by_key(&self) -> bool {
            self.left_by_key
        }
        fn ask_suppressed(&self) -> bool {
            false
        }
        fn ask_also(&self) -> bool {
            false
        }
        fn confirm_leaving(&mut self) -> Result<(), ()> {
            self.note("confirm");
            self.answer = self.answers.remove(0);
            Ok(())
        }
        fn answered_yes(&self) -> bool {
            self.answer
        }
        fn save_leave_item(&mut self) {
            self.note("save_item");
        }
    }

    const SET_UP: &[&str] = &[
        "SetUpFirst",
        "restore_highlight",
        "Prepare",
        "lists",
        "AfterLists",
        "clear_sound",
        "clear_leave",
        "Background",
        "draw",
        "setup_time",
        "BeforeFirstFrame",
        "frame_lists_first",
        "frame_first",
    ];
    const FRAME: &[&str] = &[
        "blink",
        "AfterBlink",
        "BeforeFrame",
        "frame_lists",
        "frame",
        "frame_time",
        "EachFrame",
    ];
    const LEAVE: &[&str] = &["save_item", "clear_escape", "After", "AfterRestored"];

    fn expect(parts: &[&[&str]]) -> Vec<String> {
        parts
            .iter()
            .flat_map(|part| part.iter().map(ToString::to_string))
            .collect()
    }

    #[test]
    fn a_plain_screen_left_by_escape() {
        let mut host = Mock::new(vec![Pressed::Nothing, Pressed::Escape]);
        let exit = screen_loop(&Shape::DEFAULT, &mut host).unwrap();
        assert_eq!(
            host.log,
            expect(&[
                &["Before"],
                SET_UP,
                &["AfterFirstFrame"],
                FRAME,
                &["keys"],
                FRAME,
                &["keys", "Leaving"],
                LEAVE,
            ])
        );
        assert_eq!(exit.reason, ScreenExit::Escape);
        assert_eq!((exit.frames, exit.actions), (2, 0));
    }

    #[test]
    fn an_action_that_stays_sets_up_again() {
        let mut host = Mock::new(vec![Pressed::Enter, Pressed::Escape]);
        host.results = vec![ActionResult::default()];
        let exit = screen_loop(&Shape::DEFAULT, &mut host).unwrap();
        assert_eq!(
            host.log,
            expect(&[
                &["Before"],
                SET_UP,
                &["AfterFirstFrame"],
                FRAME,
                &["keys", "keep 3", "clear_escape", "action"],
                &[
                    "clear_left",
                    "restore_action_item",
                    "clear_escape",
                    "clear_sound"
                ],
                SET_UP,
                &["AfterFirstFrame"],
                FRAME,
                &["keys", "Leaving"],
                LEAVE,
            ])
        );
        assert_eq!(exit.actions, 1);
    }

    #[test]
    fn a_plain_action_goes_to_the_next_frame() {
        let mut host = Mock::new(vec![Pressed::Enter, Pressed::Escape]);
        host.results = vec![ActionResult {
            action_mode: 2,
            ..ActionResult::default()
        }];
        screen_loop(&Shape::DEFAULT, &mut host).unwrap();
        let after_action = host.log.iter().rposition(|s| s == "clear_sound").unwrap();
        assert_eq!(host.log[after_action + 1], "blink");
    }

    #[test]
    fn an_action_that_leaves_clears_the_leave_switch() {
        let mut host = Mock::new(vec![Pressed::Enter]);
        host.results = vec![ActionResult {
            leave_screen: true,
            ..ActionResult::default()
        }];
        let exit = screen_loop(&Shape::DEFAULT, &mut host).unwrap();
        assert_eq!(
            exit.reason,
            ScreenExit::Action {
                item: 3,
                how: ActionExit::LeaveScreen { cleared: true }
            }
        );
        let tail: Vec<_> = host.log.iter().rev().take(6).rev().cloned().collect();
        assert_eq!(
            tail,
            [
                "clear_leave",
                "Leaving",
                "save_item",
                "clear_escape",
                "After",
                "AfterRestored"
            ]
        );
    }

    #[test]
    fn a_screen_that_asks_leaves_on_a_yes_and_sets_up_on_a_no() {
        let shape = Shape {
            asking: Some(AskShape {
                always: false,
                unasked_skips_save: false,
            }),
            ..Shape::DEFAULT
        };
        let mut host = Mock::new(vec![Pressed::Escape, Pressed::Escape]);
        host.left_by_key = true;
        host.answers = vec![false, true];
        screen_loop(&shape, &mut host).unwrap();
        assert_eq!(
            host.log,
            expect(&[
                &["Before"],
                SET_UP,
                &["AfterFirstFrame"],
                FRAME,
                &["keys", "AskBefore", "confirm", "OnRefusal"],
                SET_UP,
                &["AfterFirstFrame"],
                FRAME,
                &["keys", "AskBefore", "confirm", "Confirmed"],
                LEAVE,
            ])
        );
    }

    #[test]
    fn a_screen_that_leaves_at_once_runs_nothing_more() {
        let mut host = Mock::new(vec![Pressed::Escape]);
        host.leave_at_once = true;
        screen_loop(&Shape::DEFAULT, &mut host).unwrap();
        assert_eq!(host.log.last().unwrap(), "keys");
    }

    #[test]
    fn a_step_can_send_the_loop_out_and_back() {
        let mut host = Mock::new(vec![]);
        host.flows = vec![(Place::AfterFirstFrame, Flow::Leave)];
        let exit = screen_loop(&Shape::DEFAULT, &mut host).unwrap();
        assert_eq!(exit.reason, ScreenExit::OwnStep);
        assert_eq!(exit.frames, 0);
        let mut host = Mock::new(vec![Pressed::Escape]);
        host.flows = vec![(Place::AfterBlink, Flow::SetUp)];
        screen_loop(&Shape::DEFAULT, &mut host).unwrap();
        assert_eq!(host.log.iter().filter(|s| *s == "draw").count(), 2);
    }

    #[test]
    fn the_main_menu_takes_its_highlight_after_the_first_frame() {
        let shape = Shape {
            late_highlight: true,
            ..Shape::DEFAULT
        };
        let mut host = Mock::new(vec![Pressed::Escape]);
        screen_loop(&shape, &mut host).unwrap();
        assert_eq!(
            host.log[..14],
            expect(&[
                &[
                    "Before",
                    "SetUpFirst",
                    "Prepare",
                    "lists",
                    "AfterLists",
                    "Background",
                    "draw"
                ],
                &[
                    "setup_time",
                    "BeforeFirstFrame",
                    "frame_lists_first",
                    "frame_first"
                ],
                &["restore_highlight", "AfterFirstFrame"],
                &["blink"],
            ])
        );
        assert!(!host.log[..13].contains(&"clear_sound".to_string()));
    }

    #[test]
    fn a_restarting_check_clears_the_answer_when_the_event_is_over() {
        let shape = Shape {
            action_check: ActionCheck::Restart,
            ..Shape::DEFAULT
        };
        let mut host = Mock::new(vec![Pressed::Enter, Pressed::Escape]);
        host.results = vec![ActionResult {
            event_over: true,
            ..ActionResult::default()
        }];
        screen_loop(&shape, &mut host).unwrap();
        let at = host.log.iter().position(|s| s == "clear_answer").unwrap();
        assert_eq!(host.log[at + 1], "take_key");
        assert_eq!(host.log[at + 2], "SetUpFirst");
    }
}
