//! The loop of the screen where the number of players is chosen. Screen-specific part of
//! `menu::screen_loop`: background `arcade1.png`; leaving is not asked about. Screen: the player
//! count of the time trial, "ESCOLHA NÚMERO DE JOGADORES".

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, List, MenuSound, Screen, ScreenFlag, Step, carousel, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    entry.set_view(7, 10, 0x100);
    prepare_scene(&entry.owners);
    screens.flow.turning.set(0x1u32);
    screens.carousel.menu.set(0x2u32);
    carousel(entry, None)?;
    let item = screens.items.current[slot::TIME_TRIAL_SELECT_PLAYERS].get();
    screens.loop_flags.item.set(item);
    screens.carousel.front.set(item.wrapping_sub(1));
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.time_trial_select_players_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.time_trial_select_players_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.time_trial_select_players_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(ScreenFlag::EventOver, false)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.time_trial_select_players_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.time_trial_select_players.id(),
        slot::TIME_TRIAL_SELECT_PLAYERS,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::TimeTrialSelectPlayers)
}
