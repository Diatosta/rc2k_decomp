//! The loop of the visual preferences screen. Screen-specific part of `menu::screen_loop`:
//! background `visual.png`; leaving is not asked about; its own 3D scene list is set before the
//! first frame only.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Screens;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, Step, screen_entry};

fn before(screens: &Screens) -> Result<(), Refused> {
    screens.items.set_current(
        slot::VISUAL_PREFERENCES,
        screens.items.first(slot::VISUAL_PREFERENCES),
    );
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |_| DRAW_LISTS.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.visual_preferences_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.visual_preferences_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::DoScreens(before)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.visual_preferences_picture
    }))],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.visual_preferences_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.visual_preferences_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.visual_preferences_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.visual_preferences.id(),
        slot::VISUAL_PREFERENCES,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::VisualPreferences)
}
