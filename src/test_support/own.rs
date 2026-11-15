//! The owners a test holds itself: each test thread builds its own value of an owner the first
//! time the test asks for it, so tests that reach their state only through owners neither see
//! each other's writes nor wait on the `test_lock`. The values are leaked (a test thread ends
//! with the test, and `Race` and `Effects` are too large to hold in a thread-local by value).

use crate::owners::Owners;

/// Builds an owner on a thread with room for it (`Race` is 1.5 MB, a test thread has 2 MB) and
/// keeps it for the rest of the run.
fn build<T: 'static>(make: fn() -> T) -> &'static T {
    /// An owner built on a thread that has ended.
    struct Built<T>(Box<T>);
    // SAFETY: the box is the builder thread's only handle on the value, which it made fresh and
    // gives up when it ends; the value's cells are reached only from the thread that joins it
    // (the `!Send` parts are `&'static` references to statics, which the value does not share
    // with the builder thread any more).
    unsafe impl<T> Send for Built<T> {}

    let stack = size_of::<T>() * 3 + (1 << 20);
    let built = std::thread::Builder::new()
        .stack_size(stack)
        .spawn(move || Built(Box::new(make())))
        .expect("a thread to build an owner on")
        .join()
        .expect("the owner is built");
    Box::leak(built.0)
}

macro_rules! own {
    ($($name:ident: $ty:ty = $make:expr;)*) => {$(
        /// This test thread's own value of the owner.
        pub fn $name() -> &'static $ty {
            thread_local! {
                static OWNER: &'static $ty = build(|| $make);
            }
            OWNER.with(|owner| *owner)
        }
    )*};
}

own! {
    race: crate::race::owner::Race = crate::race::owner::Race::new();
    menus: crate::menu::owner::Menus = crate::menu::owner::Menus::new();
    gfx: crate::gfx::owner::Gfx = crate::gfx::owner::Gfx::new();
    strings: crate::strings::owner::Strings = crate::strings::owner::Strings::new();
    system: crate::system::owner::System = crate::system::owner::System::new();
    scene: crate::scene::owner::Scene = crate::scene::owner::Scene::new();
    saves: crate::saves::owner::Saves = crate::saves::owner::Saves::new();
    install: crate::install::owner::Install = crate::install::owner::Install::new();
    net: crate::net::owner::Net = crate::net::owner::Net::new();
    input: crate::input::owner::Input = crate::input::owner::Input::new();
    audio: crate::audio::owner::Audio = crate::audio::owner::Audio::new();
    sound: crate::sound::owner::Sound = crate::sound::owner::Sound::new();
    service: crate::service::owner::Service = crate::service::owner::Service::new();
    screens: crate::screens::owner::ScreensOwner = crate::screens::owner::ScreensOwner::new();
    hud: crate::hud::owner::Hud = crate::hud::owner::Hud::new();
    effects: crate::effects::owner::Effects = crate::effects::owner::Effects::new();
    d3d: crate::d3d::owner::D3d = crate::d3d::owner::D3d::new();
}

/// This test thread's own owners, all seventeen (building those the test has not asked for yet).
pub fn owners() -> Owners<'static> {
    Owners {
        race: race(),
        system: system(),
        scene: scene(),
        d3d: d3d(),
        sound: sound(),
        audio: audio(),
        input: input(),
        net: net(),
        gfx: gfx(),
        menus: menus(),
        effects: effects(),
        install: install(),
        saves: saves(),
        service: service(),
        strings: strings(),
        screens: screens(),
        hud: hud(),
    }
}
