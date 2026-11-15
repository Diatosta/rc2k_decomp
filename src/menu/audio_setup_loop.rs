//! The loop of the audio set-up screen. Screen-specific part of `menu::screen_loop`: background
//! `audio.png`; leaving is not asked about; its own 3D scene list is set before the first frame
//! only.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Screens;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, ScreenVar, Step, screen_entry};
use crate::menu::texts_store::texts;

fn before(screens: &Screens) -> Result<(), Refused> {
    screens.items.current[slot::AUDIO_SETUP].set(screens.items.first[slot::AUDIO_SETUP].get());
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
            table: |_| DRAW_LISTS.audio_setup_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.audio_setup_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::DoScreens(before)],
    set_up_first: &[Step::Write(ScreenVar::Quiet, 0x1)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().audio_setup_picture
    }))],
    after: &[Step::Write(ScreenVar::Quiet, 0x0)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.audio_setup_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.audio_setup_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.audio_setup_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.audio_setup.id(),
        slot::AUDIO_SETUP,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::AudioSetup)
}
