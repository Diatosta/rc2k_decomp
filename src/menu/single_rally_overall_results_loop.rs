//! The loop of a results screen, by its title "RESULTADOS GERAIS". Screen-specific part of
//! `menu::screen_loop`: background `results1.png` through `menu::load_background`; leaving is not
//! asked about; its own 3D scene list is set before the first frame only.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |m| m.tables.draw_lists.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.single_rally_overall_results_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| {
                m.tables
                    .draw_lists
                    .single_rally_overall_results_overlay
                    .id()
            },
            run: false,
        },
    ],
    before: &[Step::WriteFlag(|m| &m.screens.event.event_over, false)],
    set_up_first: &[Step::Call(Callee::RankTotals)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.single_rally_overall_results_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |m| m.tables.draw_lists.single_rally_overall_results_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |m| m.tables.draw_lists.single_rally_overall_results_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |m| {
                    m.tables
                        .draw_lists
                        .single_rally_overall_results_overlay
                        .id()
                },
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.single_rally_overall_results.id(),
        |m| &m.screens.items.current[slot::SINGLE_RALLY_OVERALL_RESULTS],
    )
};
