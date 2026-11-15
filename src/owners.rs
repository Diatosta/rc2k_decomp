//! The game's owners handed down together: one reference to each module's state, for the
//! functions that reach several of them (`Owners`). `Owners::with_menus` is made once, at the
//! root of the game's thread; the slot threads have `thread_slot::SlotOwners`. A root that
//! cannot take owners from a caller (a thread's or a platform callback's) is marked `m6-root`
//! (`tools/m6_count.py` leaves it out).

use crate::audio::owner::Audio;
use crate::d3d::owner::D3d;
use crate::effects::owner::Effects;
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
    pub d3d: &'a D3d,
    pub sound: &'a Sound,
    pub audio: &'a Audio,
    pub input: &'a Input,
    pub net: &'a Net,
    pub gfx: &'a Gfx,
    pub menus: &'a Menus,
    pub effects: &'a Effects,
    pub install: &'a Install,
    pub saves: &'a Saves,
    pub strings: &'a Strings,
}

impl Owners<'static> {
    /// The tests' owners: the shared static menus and the other owners' accessors.
    #[cfg(test)]
    pub fn the() -> Self {
        Self::with_menus(crate::menu::owner::menus())
    }

    /// The game's owners: the menus the root made and the other owners, as their accessors give
    /// them (those held by the game's thread through that thread's `get()`).
    pub fn with_menus(menus: &'static Menus) -> Self {
        Owners {
            race: crate::race::owner::race(),
            system: crate::system::owner::system(),
            scene: crate::scene::owner::scene(),
            d3d: crate::d3d::owner::d3d(),
            sound: crate::sound::owner::sound(),
            audio: crate::audio::owner::audio(),
            input: crate::input::owner::input(),
            net: crate::net::owner::net(),
            gfx: crate::gfx::owner::gfx(),
            menus,
            effects: crate::effects::owner::effects(),
            install: crate::install::owner::install(),
            saves: crate::saves::owner::saves(),
            strings: crate::strings::owner::strings(),
        }
    }
}
