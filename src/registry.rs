use crate::framework::GameHook;
use crate::{
    audio, d3d, drive, fatal, files, gfx, hud, input, install, logfile, marks, math, memory, menu,
    net, physics, race, records, saves, scene, screens, service, settings, sound, strings, system,
    text, threading,
};
use std::sync::LazyLock;

/// Whenever a hook module is registered, it is added to this list.
pub static ALL: LazyLock<Vec<&'static dyn GameHook>> = LazyLock::new(|| {
    [
        threading::HOOKS,
        strings::HOOKS,
        math::HOOKS,
        records::HOOKS,
        text::HOOKS,
        gfx::HOOKS,
        &gfx::mmx_converters::HOOKS,
        &input::effect_objects::HOOKS,
        screens::HOOKS,
        service::HOOKS,
        input::HOOKS,
        menu::HOOKS,
        &menu::item_actions::HOOKS,
        &menu::ranked_lookups::HOOKS,
        &menu::player_actions::HOOKS,
        &install::install_frames::HOOKS,
        &screens::results_pages::HOOKS,
        &menu::learn_bindings::HOOKS,
        &menu::option_items::HOOKS,
        saves::HOOKS,
        system::HOOKS,
        fatal::HOOKS,
        files::HOOKS,
        logfile::HOOKS,
        memory::HOOKS,
        settings::HOOKS,
        install::HOOKS,
        physics::HOOKS,
        sound::HOOKS,
        audio::HOOKS,
        d3d::HOOKS,
        scene::HOOKS,
        race::HOOKS,
        marks::HOOKS,
        drive::HOOKS,
        hud::HOOKS,
        net::HOOKS,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect()
});

/// Whether a name or prefix typed in the console selects the hook called `name`: `all` and a
/// prefix select every hook they match, but the full name of a hook selects only that hook,
/// even if other names start with it.
pub fn selects(pattern: &str, name: &str) -> bool {
    if ALL.iter().any(|h| h.name() == pattern) {
        return name == pattern;
    }
    pattern == "all" || name.starts_with(pattern)
}
