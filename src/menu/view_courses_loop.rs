//! The loop of the course information screen. Screen-specific part of `menu::screen_loop`: leaving
//! is not asked about.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, List, Screen, ScreenVar, Step, log_line, screen_entry,
};

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    log_line(c"VCoursesScr: Start");
    Ok(())
}

fn after(_: &Entry) -> Result<(), Refused> {
    log_line(c"VCoursesScr: End");
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.view_courses_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.view_courses_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.view_courses_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.view_courses_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::Write(ScreenVar::StageMode, 0x1),
        Step::Write(ScreenVar::StageGroup, 0x0),
        Step::Write(ScreenVar::StageNumber, 0x1),
        Step::Write(ScreenVar::StageSub, 0x0),
        Step::Write(ScreenVar::ArcadeLevel, 0x0),
        Step::Write(ScreenVar::ArcadeStage, 0x1),
        Step::Write(ScreenVar::EventSection, 0x0),
        Step::Write(ScreenVar::EventIndex, 0x0),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after: &[Step::Do(after)],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.view_courses.id(),
        slot::VIEW_COURSES,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::ViewCourses)
}
