//! The game's owners handed down together: one reference to each module's state, for the
//! functions that reach several of them (`Owners`). `Owners::build` is made once, at the
//! root of the game's thread; the slot threads have `thread_slot::SlotOwners`. A root that
//! cannot take owners from a caller (a thread's or a platform callback's) is marked `m6-root`
//! (`tools/m6_count.py` leaves it out).

use crate::audio::owner::Audio;
use crate::gfx::owner::Gfx;
use crate::input::owner::Input;
use crate::install::owner::Install;
use crate::menu::owner::Menus;
use crate::net::owner::Net;
use crate::race::owner::Race;
use crate::saves::owner::Saves;
use crate::scene::owner::Scene;
use crate::sound::owner::Sound;
use crate::strings::owner::Strings;
use crate::system::owner::System;

/// A reference to each owner. Copy it down; a function takes the narrowest owner it needs.
#[derive(Clone, Copy)]
pub struct Owners<'a> {
    pub race: &'a Race,
    pub system: &'a System,
    pub scene: &'a Scene,
    pub sound: &'a Sound,
    pub audio: &'a Audio,
    pub input: &'a Input,
    pub net: &'a Net,
    pub gfx: &'a Gfx,
    pub menus: &'a Menus,
    pub install: &'a Install,
    pub saves: &'a Saves,
    pub strings: &'a Strings,
}

impl Owners<'static> {
    /// The tests' owners: this test thread's own, built the first time it asks.
    #[cfg(test)]
    pub fn the() -> Self {
        crate::test_support::own::owners()
    }

    /// Builds every owner once, for the rest of the run: they live on the heap and are never
    /// freed (the slot threads and the fatal reporter outlive the game thread's frames). They
    /// are made on a thread with room for them (`Race` alone is 1.5 MB).
    pub fn build() -> Self {
        on_big_stack(
            3 * (size_of::<Race>()
                + size_of::<System>()
                + size_of::<Scene>()
                + size_of::<Net>()
                + size_of::<Audio>()
                + size_of::<Gfx>()
                + size_of::<Menus>()
                + size_of::<Saves>()
                + size_of::<Strings>()
                + size_of::<Install>()
                + size_of::<Sound>()
                + size_of::<Input>())
                + (1 << 20),
            || Owners {
                race: leak(Race::new()),
                system: leak(System::new()),
                scene: leak(Scene::new()),
                sound: leak(Sound::new()),
                audio: leak(Audio::new()),
                input: leak(Input::new()),
                net: leak(Net::new()),
                gfx: leak(Gfx::new()),
                menus: leak(Menus::new()),
                install: leak(Install::new()),
                saves: leak(Saves::new()),
                strings: leak(Strings::new()),
            },
        )
    }
}

/// A value kept for the rest of the run.
fn leak<T>(value: T) -> &'static T {
    Box::leak(Box::new(value))
}

/// Runs `make` on a thread with `stack` bytes of stack and gives back what it made: the owners
/// are too large for the game thread's stack while they are built (a value is made on the stack
/// before it is boxed).
///
/// `make` must build its result fresh and keep no other handle on it, so that the thread that
/// joins the builder is the result's only user (a closure that is `Send` and holds no shared
/// handle to what it makes).
pub fn on_big_stack<R: 'static>(stack: usize, make: impl FnOnce() -> R + Send + 'static) -> R {
    /// A value made on a thread that has ended.
    struct Made<R>(R);
    #[allow(
        unsafe_code,
        reason = "the one hand-over of a freshly built owner to the thread that joins its builder"
    )]
    // SAFETY: the value is made fresh by a closure that holds no handle on it and goes to the thread
    // that joins the builder, which ends with it: no handle stays behind, so the `!Send` parts
    // (cells, references to leaked values) are reached by one thread at a time.
    unsafe impl<R> Send for Made<R> {}

    std::thread::Builder::new()
        .stack_size(stack)
        .spawn(move || Made(make()))
        .expect("a thread to build the owners on")
        .join()
        .expect("the owners are built")
        .0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The game's owners are built apart from a test thread's own.
    #[test]
    fn the_game_owners_are_built_apart_from_the_tests_own() {
        let built = Owners::build();
        let mine = Owners::the();
        assert!(!std::ptr::eq(built.race, mine.race));
        assert!(!std::ptr::eq(built.menus, mine.menus));
    }
}
