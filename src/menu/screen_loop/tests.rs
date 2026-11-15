//! Every screen's description, resolved and run on a `Menus` of its own: the registry is
//! complete and consistent, the data steps write what they name and nothing else, and the loop
//! runs each screen's shape and data steps to its leaving with the platform replaced by a
//! script.

use super::*;
use crate::menu::screen_items_store::SLOTS;
use crate::test_support::test_lock;
use rc2k_core::menu::screen_loop::{Place, ScreenHost};

/// A `Menus` that nothing else reaches.
fn mock_menus() -> &'static Menus {
    Box::leak(Box::new(Menus::new()))
}

/// Every place of the loop, in the order the loop first reaches them.
const PLACES: [Place; 16] = [
    Place::Before,
    Place::SetUpFirst,
    Place::Prepare,
    Place::AfterLists,
    Place::Background,
    Place::BeforeFirstFrame,
    Place::AfterFirstFrame,
    Place::AfterBlink,
    Place::BeforeFrame,
    Place::EachFrame,
    Place::AskBefore,
    Place::OnRefusal,
    Place::Confirmed,
    Place::Leaving,
    Place::After,
    Place::AfterRestored,
];

/// Every step of the screen, with the place it runs at.
fn steps_of(screen: &Screen) -> Vec<(Place, &'static [Step])> {
    PLACES
        .iter()
        .map(|&place| (place, steps_at(screen, place)))
        .collect()
}

/// `Debug` of a variant in snake case.
fn snake(name: &str) -> String {
    let mut out = String::new();
    for (at, c) in name.chars().enumerate() {
        if c.is_uppercase() && at > 0 {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

#[test]
fn the_registry_names_every_screen_once() {
    assert_eq!(ScreenId::ALL.len(), 85);
    let mut paths = std::collections::HashSet::new();
    for id in ScreenId::ALL {
        let path = id.screen().module_path;
        assert!(
            paths.insert(path),
            "{id:?} shares its file with another screen"
        );
        let file = path.rsplit("::").next().unwrap();
        let suffix = if path.contains("::player_pages::") {
            ""
        } else {
            "_loop"
        };
        assert_eq!(
            format!("{}{suffix}", snake(&format!("{id:?}"))),
            file,
            "{id:?}"
        );
    }
}

#[test]
fn every_name_in_a_description_resolves() {
    let menus = mock_menus();
    for id in ScreenId::ALL {
        let screen = id.screen();
        (screen.menu)(menus);
        let slots = [
            Some(screen.saved_item),
            screen.keys.action_saved_item,
            screen.leave_saved_item,
        ];
        for slot in slots.into_iter().flatten() {
            assert!(slot < SLOTS, "{id:?}: item slot {slot}");
        }
        if let Some(other) = screen.keys.action_menu {
            other(menus);
        }
        let first = screen.frames.first_frame_lists.unwrap_or(&[]);
        for list in screen.lists.iter().chain(first) {
            let table = (list.table)(menus);
            table.functions();
            (list.pointer)(menus).set(table);
            assert!((list.pointer)(menus).get().is_ok(), "{id:?}: list pointer");
        }
        for (place, steps) in steps_of(screen) {
            for step in steps {
                if let Step::Call(Callee::LoadBackground(path) | Callee::LoadPicture(path)) = step {
                    path(menus);
                }
                if let Step::Write(ScreenVar::SavedItem(slot), _) = step {
                    assert!(*slot < SLOTS, "{id:?} {place:?}: item slot {slot}");
                }
            }
        }
    }
}

const VARS: [ScreenVar; 17] = [
    ScreenVar::ArcadeLevel,
    ScreenVar::ArcadeStage,
    ScreenVar::StageMode,
    ScreenVar::NoticeDue,
    ScreenVar::ActionMode,
    ScreenVar::ClearsEscapeFlag,
    ScreenVar::CurrentPlayer,
    ScreenVar::EventIndex,
    ScreenVar::EventSection,
    ScreenVar::ServiceNext,
    ScreenVar::StageGroup,
    ScreenVar::StageNumber,
    ScreenVar::StageSub,
    ScreenVar::LeftService,
    ScreenVar::Quiet,
    ScreenVar::Highlight,
    ScreenVar::YesNoAnswer,
];

const FLAGS: [ScreenFlag; 10] = [
    ScreenFlag::EscapePressed,
    ScreenFlag::EventOver,
    ScreenFlag::Asking,
    ScreenFlag::AskLeave,
    ScreenFlag::EventLeft,
    ScreenFlag::LeaveScreen,
    ScreenFlag::LeftWithCarry,
    ScreenFlag::SkipEditValues,
    ScreenFlag::StandingsShown,
    ScreenFlag::LeftByKey,
];

const SCENE_VARS: [SceneVar; 11] = [
    SceneVar::ViewOffsetX,
    SceneVar::ViewOffsetY,
    SceneVar::ViewZoom,
    SceneVar::ClipLeft,
    SceneVar::ClipRight,
    SceneVar::LightLowest,
    SceneVar::LightHighest,
    SceneVar::LightLevel,
    SceneVar::LightTurn0,
    SceneVar::LightTurn1,
    SceneVar::ObjectLightStrength,
];

const SCENE_FLOATS: [SceneFloat; 2] = [SceneFloat::ClipLeft, SceneFloat::ClipRight];

/// Every writable variable's value.
#[derive(Debug, PartialEq)]
struct Snapshot {
    vars: Vec<u32>,
    flags: Vec<bool>,
    slots: Vec<u32>,
    sound: MenuSound,
    scene: Vec<u32>,
    floats: Vec<u32>,
}

fn snapshot(menus: &Menus) -> Snapshot {
    Snapshot {
        vars: VARS.iter().map(|v| v.cell(menus).get()).collect(),
        flags: FLAGS.iter().map(|f| f.cell(menus).is_set()).collect(),
        slots: (0..SLOTS)
            .map(|slot| ScreenVar::SavedItem(slot).cell(menus).get())
            .collect(),
        sound: menus.screens.entry.sound(),
        scene: SCENE_VARS.iter().map(|v| v.cell().get()).collect(),
        floats: SCENE_FLOATS
            .iter()
            .map(|v| v.cell().get().to_bits())
            .collect(),
    }
}

/// Puts a different value in every variable, so that a write shows.
fn fill(menus: &Menus) {
    for (at, var) in VARS.iter().enumerate() {
        var.cell(menus).set(0x5000 + at as u32);
    }
    for slot in 0..SLOTS {
        ScreenVar::SavedItem(slot)
            .cell(menus)
            .set(0x6000 + slot as u32);
    }
    for (at, var) in SCENE_VARS.iter().enumerate() {
        var.cell().set(0x7000 + at as u32);
    }
    for (at, var) in SCENE_FLOATS.iter().enumerate() {
        var.cell().set(0.5 + at as f32);
    }
    for flag in FLAGS {
        flag.cell(menus).set(false);
    }
    menus.screens.entry.set_sound(MenuSound::Enter);
}

#[test]
fn variables_name_distinct_places() {
    let menus = mock_menus();
    let mut places: Vec<*const ()> = VARS
        .iter()
        .map(|v| std::ptr::from_ref(v.cell(menus)).cast())
        .collect();
    places.extend((0..SLOTS).map(|slot| -> *const () {
        std::ptr::from_ref(ScreenVar::SavedItem(slot).cell(menus)).cast()
    }));
    places.extend(
        FLAGS
            .iter()
            .map(|f| -> *const () { std::ptr::from_ref(f.cell(menus)).cast() }),
    );
    let count = places.len();
    places.sort();
    places.dedup();
    assert_eq!(places.len(), count);
}

/// Every data step of every screen changes the variable it names and no other.
#[test]
fn data_steps_write_what_they_name_and_nothing_else() {
    let _lock = test_lock();
    let menus = mock_menus();
    let saved = snapshot(menus);
    let mut run_steps = 0;
    for id in ScreenId::ALL {
        for (place, steps) in steps_of(id.screen()) {
            for step in steps {
                fill(menus);
                let before = snapshot(menus);
                if !step.write(menus) {
                    assert_eq!(snapshot(menus), before, "{id:?} {place:?}: {step:?}");
                    continue;
                }
                run_steps += 1;
                let mut after = snapshot(menus);
                // Put the step's own target back to what it was before: the rest must match.
                match *step {
                    Step::Write(ScreenVar::SavedItem(slot), value) => {
                        assert_eq!(after.slots[slot], value);
                        after.slots[slot] = before.slots[slot];
                    }
                    Step::Write(var, value) => {
                        let at = VARS.iter().position(|&v| v == var).unwrap();
                        assert_eq!(after.vars[at], value);
                        after.vars[at] = before.vars[at];
                    }
                    Step::WriteFlag(flag, on) => {
                        let at = FLAGS.iter().position(|&f| f == flag).unwrap();
                        assert_eq!(after.flags[at], on);
                        after.flags[at] = before.flags[at];
                    }
                    Step::Sound(sound) => {
                        assert_eq!(after.sound, sound);
                        after.sound = before.sound;
                    }
                    Step::WriteScene(var, value) => {
                        let at = SCENE_VARS.iter().position(|&v| v == var).unwrap();
                        assert_eq!(after.scene[at], value);
                        after.scene[at] = before.scene[at];
                    }
                    Step::WriteSceneF32(var, value) => {
                        let at = SCENE_FLOATS.iter().position(|&v| v == var).unwrap();
                        assert_eq!(f32::from_bits(after.floats[at]), value);
                        after.floats[at] = before.floats[at];
                    }
                    Step::Call(_) | Step::CallOrFatal(..) | Step::Do(_) | Step::DoFlow(_) => {
                        unreachable!()
                    }
                }
                assert_eq!(after, before, "{id:?} {place:?}: {step:?} wrote elsewhere");
            }
        }
    }
    assert!(run_steps > 100, "{run_steps} data steps");
    // The scene variables are the game's own statics: put them back.
    for (var, value) in SCENE_VARS.iter().zip(&saved.scene) {
        var.cell().set(*value);
    }
    for (var, bits) in SCENE_FLOATS.iter().zip(&saved.floats) {
        var.cell().set(f32::from_bits(*bits));
    }
}

/// The screen loop's host on a `Menus` of its own: the loop's own bookkeeping is the real one
/// (`MenuHost`), the places that reach the platform are scripted, and a screen's own steps run
/// only where they write variables.
struct ScriptedHost<'a> {
    real: MenuHost<'a>,
    keys: Vec<Pressed>,
    places: Vec<Place>,
}

impl ScreenHost for ScriptedHost<'_> {
    type Error = Refused;

    fn run(&mut self, place: Place) -> Result<Flow, Refused> {
        self.places.push(place);
        for step in steps_at(self.real.screen, place) {
            step.write(self.real.entry.menus);
        }
        Ok(Flow::On)
    }
    fn restore_highlight(&mut self) {
        self.real.restore_highlight();
    }
    fn put_menu_and_lists(&mut self) -> Result<(), Refused> {
        let menus = self.real.entry.menus;
        self.real.screens().loop_flags.menu.set(self.real.menu);
        for list in self.real.screen.lists {
            (list.pointer)(menus).set((list.table)(menus));
        }
        Ok(())
    }
    fn clear_menu_sound(&mut self) {
        self.real.clear_menu_sound();
    }
    fn clear_leave_screen(&mut self) {
        self.real.clear_leave_screen();
    }
    fn draw_set_up_frame(&mut self) -> Result<(), Refused> {
        Ok(())
    }
    fn note_set_up_time(&mut self) {}
    fn put_frame_lists(&mut self, first: bool) {
        self.real.put_frame_lists(first);
    }
    fn run_frame(&mut self, _first: bool) -> Result<(), Refused> {
        Ok(())
    }
    fn update_blink(&mut self) {}
    fn escape_flag(&self) -> bool {
        self.real.escape_flag()
    }
    fn quitting(&self) -> bool {
        false
    }
    fn leave_on_close(&mut self) -> ! {
        panic!("the window was closed");
    }
    fn debug_serve(&mut self) -> Result<bool, Refused> {
        Ok(false)
    }
    fn note_frame_time(&mut self) {}
    fn handle_keys(&mut self) -> Result<Pressed, Refused> {
        let key = self.keys.remove(0);
        let menus = self.real.entry.menus;
        menus.screens.entry.set_sound(if key == Pressed::Enter {
            MenuSound::Enter
        } else {
            MenuSound::None
        });
        Ok(key)
    }
    fn action_pending(&self) -> bool {
        self.real.action_pending()
    }
    fn highlight(&self) -> u32 {
        self.real.highlight()
    }
    fn keep_action_item(&mut self, item: u32) {
        self.real.keep_action_item(item);
    }
    fn clear_escape(&mut self) -> bool {
        self.real.clear_escape()
    }
    fn run_item_action(&mut self) -> Result<ActionResult, Refused> {
        // An action that leaves the screen in place and asks nothing.
        Ok(ActionResult::default())
    }
    fn clear_left_by_key(&mut self) {
        self.real.clear_left_by_key();
    }
    fn restore_action_item(&mut self) {
        self.real.restore_action_item();
    }
    fn clear_answer(&mut self) {
        self.real.clear_answer();
    }
    fn take_key(&mut self) -> Result<(), Refused> {
        Ok(())
    }
    fn leave_at_once(&self) -> bool {
        self.real.leave_at_once()
    }
    fn left_by_key(&self) -> bool {
        self.real.left_by_key()
    }
    fn ask_suppressed(&self) -> bool {
        false
    }
    fn ask_also(&self) -> bool {
        self.real.ask_also()
    }
    fn confirm_leaving(&mut self) -> Result<(), Refused> {
        // The question is answered yes.
        self.real.entry.menus.tables.pages.yes_no_answer.set(1u32);
        Ok(())
    }
    fn answered_yes(&self) -> bool {
        self.real.answered_yes()
    }
    fn save_leave_item(&mut self) {
        self.real.save_leave_item();
    }
}

/// Runs `screen` on a fresh `Menus` with `keys` for its frames; the places it went through,
/// the loop's exit, and the menus.
fn run_scripted(id: ScreenId, keys: Vec<Pressed>) -> (Vec<Place>, Exit, &'static Menus) {
    let menus = mock_menus();
    let entry = Entry {
        menus,
        ..Entry::new(crate::owners::Owners::the())
    };
    let screen = id.screen();
    let menu = (screen.menu)(menus);
    let mut host = ScriptedHost {
        real: MenuHost {
            screen,
            entry: &entry,
            menu,
            action_menu: screen.keys.action_menu.map_or(menu, |other| other(menus)),
        },
        keys,
        places: Vec::new(),
    };
    // The saved highlight comes back at set-up and is saved again on leaving.
    ScreenVar::SavedItem(screen.saved_item).cell(menus).set(5);
    let exit = core_loop(&screen.shape(), &mut host)
        .unwrap_or_else(|_| panic!("{id:?}: the loop refused"));
    (host.places, exit, menus)
}

/// With a key for the first frame and Escape for the second, every screen's loop reaches its
/// leaving, runs its places in the loop's order, and keeps the highlight in its own slot.
#[test]
fn every_screen_runs_to_its_leaving_by_escape() {
    let _lock = test_lock();
    for id in ScreenId::ALL {
        let screen = id.screen();
        let (places, exit, menus) = run_scripted(id, vec![Pressed::Nothing, Pressed::Escape]);
        assert_eq!(places[0], Place::Before, "{id:?}");
        // A screen whose "next frame" repeats its first frame's steps runs `AfterFirstFrame`
        // again.
        let mut first_pass: Vec<Place> = places
            .iter()
            .copied()
            .filter(|p| {
                matches!(
                    p,
                    Place::SetUpFirst
                        | Place::Prepare
                        | Place::AfterLists
                        | Place::Background
                        | Place::BeforeFirstFrame
                        | Place::AfterFirstFrame
                )
            })
            .collect();
        first_pass.dedup();
        assert_eq!(
            first_pass,
            [
                Place::SetUpFirst,
                Place::Prepare,
                Place::AfterLists,
                Place::Background,
                Place::BeforeFirstFrame,
                Place::AfterFirstFrame,
            ],
            "{id:?}: set-up places {places:?}"
        );
        assert_eq!(exit.actions, 0, "{id:?}");
        if screen.asking.is_none() && screen.leave_at_once.is_none() {
            assert_eq!(places.last(), Some(&Place::AfterRestored), "{id:?}");
            let kept = screen.leave_saved_item.unwrap_or(screen.saved_item);
            assert_eq!(
                ScreenVar::SavedItem(kept).cell(menus).get(),
                5,
                "{id:?}: highlight"
            );
        }
    }
}

/// An action that stays on the screen sets it up again and the loop then leaves by Escape.
#[test]
fn every_screen_sets_up_again_after_an_action_that_stays() {
    let _lock = test_lock();
    for id in ScreenId::ALL {
        let screen = id.screen();
        let (places, exit, _) = run_scripted(id, vec![Pressed::Enter, Pressed::Escape]);
        assert_eq!(exit.actions, 1, "{id:?}: {exit}");
        let set_ups = places.iter().filter(|p| **p == Place::Background).count();
        if screen.keys.action_check == ActionCheck::Standard && screen.asking.is_none() {
            assert_eq!(set_ups, 2, "{id:?}: {places:?}");
        }
    }
}
