//! The players' profiles (`rc2k_core::input::profile`) at run time: one for each controller a
//! player has used, made the first time the player needs it, and what the race and the
//! controller pages do with them.
//!
//! The race sets a device up from a car's controls; the car's controller number is a place in
//! the controllers' table, the player the record the race took the controls from
//! (`Input::race_players`), so the profile is the one of that player for the controller the
//! place names. The device then reads through a copy of it taken at the set-up.

use crate::input::owner::Input;
use rc2k_core::input::device_id::id_of;
use rc2k_core::input::profile::{PROFILES_MOST, Profile, RawInput, use_profile};
use std::sync::MutexGuard;
use std::sync::atomic::Ordering;

fn all(input: &Input) -> MutexGuard<'_, [Vec<Profile>; 4]> {
    input
        .profiles
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The profiles of the four players, for the options file.
pub fn kept(input: &Input) -> [Vec<Profile>; 4] {
    all(input).clone()
}

/// The profiles from the options file.
pub fn keep(input: &Input, mut profiles: [Vec<Profile>; 4]) {
    profiles
        .iter_mut()
        .flatten()
        .for_each(rc2k_core::input::touch::give_lights);
    // A file with more than a player keeps (an older one) loses the least recently used first.
    for list in &mut profiles {
        let extra = list.len().saturating_sub(PROFILES_MOST);
        list.drain(..extra);
    }
    *all(input) = profiles;
}

/// What the profile of `controller` (1 for the table's first, as the player's record has it)
/// needs to be made: the controller as it is kept, and what kind it is.
fn identity(
    input: &Input,
    controller: u32,
) -> Option<(
    rc2k_core::input::device_id::DeviceId,
    crate::input::di_objects::DeviceInfo,
)> {
    let at = controller.checked_sub(1)? as usize;
    let id = id_of(&crate::input::device_ids::listed(input), at)?;
    let entry = input.devices.controllers.get(at)?;
    let info = crate::input::di_objects::device_info(&entry.instance.read())?;
    Some((id, info))
}

/// Player `player`'s profile for the controller at place `controller` of the table (the
/// player's own, made with its defaults if the player has none for it), `f` applied to it; none
/// when there is no such controller.
pub fn with_profile<R>(
    input: &Input,
    player: u32,
    controller: u32,
    f: impl FnOnce(&mut Profile) -> R,
) -> Option<R> {
    let (id, info) = identity(input, controller)?;
    let mut all = all(input);
    let list = all.get_mut(player as usize)?;
    Some(f(use_profile(list, &id, info.kind)))
}

/// What player `player`'s buttons on the controller at place `controller` are drawn with: the
/// pictures (none when the player has them as numbers, or the controller is none or unknown to
/// the program) and the profile's bindings (for the menu controls).
pub fn pictures(
    input: &Input,
    player: u32,
    controller: u32,
) -> Option<(
    rc2k_core::input::icons::Pictures,
    rc2k_core::input::profile::Bindings,
)> {
    let (_, info) = identity(input, controller)?;
    let (style, bindings) = with_profile(input, player, controller, |p| {
        (p.button_icons, p.bindings.clone())
    })?;
    let pictures = rc2k_core::input::icons::Pictures::of(info.kind, style, info.icon_family)?;
    Some((pictures, bindings))
}

/// The pictures for what the race device `device` reads through, with the bindings it reads by
/// (none for a keyboard, buttons shown as numbers, or a controller the program does not know).
pub fn race_pictures(
    input: &Input,
    device: usize,
) -> Option<(
    rc2k_core::input::icons::Pictures,
    rc2k_core::input::profile::Bindings,
)> {
    let profile = race_profile(input, device)?;
    // The record's controller is a place in the table counted from 0; the profiles count from 1.
    let controller = input.devices.device(device)?.controller.get() + 1;
    let (_, info) = identity(input, controller)?;
    let pictures =
        rc2k_core::input::icons::Pictures::of(info.kind, profile.button_icons, info.icon_family)?;
    Some((pictures, profile.bindings))
}

/// What a device record reads through in the race: set at the race's start.
pub(super) fn set_race_profile(input: &Input, device: usize, profile: Option<Profile>) {
    let mut held = input
        .race_profiles
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(slot) = held.get_mut(device) {
        *slot = profile;
    }
}

/// The player whose controls car `device` took.
pub(super) fn race_player(input: &Input, device: usize) -> u32 {
    input
        .race_players
        .get(device)
        .map_or(0, |p| p.load(Ordering::Relaxed))
}

/// Whether the player on race device `device` has the engine's part of the force feedback on
/// (on for a device with no profile: a keyboard has no feedback to take it from).
pub(super) fn race_engine_vibration(input: &Input, device: usize) -> bool {
    race_profile(input, device).is_none_or(|p| p.engine_vibration)
}

/// The profile device `device` reads through.
pub(super) fn race_profile(input: &Input, device: usize) -> Option<Profile> {
    input
        .race_profiles
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(device)?
        .clone()
}

/// The menu bindings of the controller called `name`, from the first player's profile for it that
/// binds any (a controller in the menus belongs to no player: whoever has used it). None when no
/// profile for it binds a menu control: its defaults then.
pub fn menu_map_of(input: &Input, name: &str) -> Option<rc2k_core::input::profile::MenuMap> {
    use rc2k_core::input::profile::{MenuControl, MenuMap};
    all(input)
        .iter()
        .flatten()
        .filter(|p| p.device.name == name)
        .map(MenuMap::of)
        .find(|map| MenuControl::ALL.iter().any(|&c| map.is_bound(c)))
}

/// What the first device reads now, as the profiles take it.
pub fn first_reading(input: &Input) -> Option<RawInput> {
    let record = input.devices.device(0)?;
    crate::input::di_objects::profile_reading(record.device.get())
}

/// The controller's bindings page was entered for `player`: their profile for the controller they
/// have chosen is kept to be put back by `undo`.
pub fn remember_for_undo(input: &Input, player: u32, controller: u32) {
    let kept = with_profile(input, player, controller, |p| (player, p.clone()));
    *input
        .profile_undo
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = kept;
}

/// The player's profile for their controller put back as it was on entering the page.
pub fn undo(input: &Input, player: u32, controller: u32) {
    let kept = input
        .profile_undo
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    if let Some((who, saved)) = kept
        && who == player
    {
        with_profile(input, player, controller, |p| {
            if p.device == saved.device {
                *p = saved;
            }
        });
    }
}

/// How the profiles number a controller's axes, and what kind of controller it is (none when
/// there is no controller at that place).
pub fn controller_kind(
    input: &Input,
    controller: u32,
) -> Option<(rc2k_core::input::profile::Kind, usize)> {
    let (_, info) = identity(input, controller)?;
    Some((info.kind, info.profile_axes as usize))
}

/// The controller's reading now, kept as the rest the learning measures from.
pub fn begin_learning(input: &Input) {
    input
        .learn_back_since
        .store(0, std::sync::atomic::Ordering::Relaxed);
    *input
        .learn_rest
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = first_reading(input);
}

/// The reading the learning measures from (taken now if there is none yet). A button or a hat
/// held when the learning began counts once it has been let go and pressed again, so the
/// press that started the learning is not the answer: the rest follows what is released.
pub fn learning_rest(input: &Input, now: &RawInput) -> RawInput {
    let mut held = input
        .learn_rest
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let rest = held.get_or_insert(*now);
    rest.buttons &= now.buttons;
    for (rest_hat, &hat) in rest.hats.iter_mut().zip(&now.hats) {
        if hat >= 36000 {
            *rest_hat = hat;
        }
    }
    *rest
}
