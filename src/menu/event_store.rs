//! The event's state, out of RAL.EXE's image (IDA 0x7558C4 .. 0x7559A4; phase 7, typed): the stage
//! chosen, the classes, the times and flags of the event's players, the service, the event's
//! type, kind and sections, the current player and the car shown. A `#[repr(C)]` struct with the
//! image's layout, one static; `moved::MOVED` points the range at it. (Split 2026-10-25 from the front
//! end's block `menu::store`.)

use crate::session::{Bytes, Dword, with_runs};

/// The event's state (IDA 0x7558C4 .. 0x7559A4).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Event {
    /// `dword_7558C4`: Not worked out.
    pub dword_7558c4: Dword,
    /// `dword_7558C8`: The stage chosen: its group, 0 or 1 (`menu::stage_choice`).
    pub stage_group: Dword,
    /// `dword_7558CC`: The stage chosen: its number in its group, 1 .. 6.
    pub stage_number: Dword,
    /// `dword_7558D0`: The stage chosen: its sub-group, 0 .. 5.
    pub stage_sub: Dword,
    /// `| Not worked out.`:
    pub dword_7558d4: Dword,
    /// `g_clearsEscapeFlag`: 1 clears the Escape flag with the highlight (`menu::screen_loop`).
    pub clears_escape_flag: Dword,
    /// `dword_7558DC`: The `sparksmode` setting of `var\ral.cfg`.
    pub sparks_mode: Dword,
    /// `dword_7558E0`: Not worked out.
    pub dword_7558e0: Dword,
    /// `dword_7558E4`: Not worked out.
    pub dword_7558e4: Dword,
    /// `dword_7558E8`: Not worked out.
    pub dword_7558e8: Dword,
    /// `dword_7558EC`: Not worked out.
    pub dword_7558ec: Dword,
    /// `dword_7558F0`: Not worked out.
    pub dword_7558f0: Dword,
    /// `dword_7558F4`: The class shown on the class pages.
    pub class: Dword,
    /// `dword_7558F8`: The number of classes (`menu::calc_cpu_times`).
    pub classes: Dword,
    /// `| Not worked out.`:
    pub dword_7558fc: Dword,
    /// `dword_755900`: Not worked out.
    pub dword_755900: Dword,
    /// `dword_755904`: Not worked out.
    pub dword_755904: Dword,
    /// `dword_755908`: Not worked out.
    pub dword_755908: Dword,
    /// `dword_75590C`: Not worked out.
    pub dword_75590c: Dword,
    /// `dword_755910`: Not worked out.
    pub dword_755910: Dword,
    /// `g_playerTime`: The player's time (`menu::time_trial_qualifies`).
    pub player_time: Dword,
    /// `g_timeQualifies`: 1 when the time qualifies / beats the target.
    pub time_qualifies: Dword,
    /// `| Not worked out.`:
    pub dword_75591c: Dword,
    /// `g_anyTimeQualified`: 1 when any player's time qualified.
    pub any_time_qualified: Dword,
    /// `dword_755924`: The welcome page's figure (`menu::run_championship`).
    pub welcome_figure: Dword,
    /// `g_championshipEnd`: The championship is over.
    pub championship_end: Dword,
    /// `g_eventGoesOn`: 1 while the event goes on.
    pub event_goes_on: Dword,
    /// `dword_755930`: Not worked out.
    pub dword_755930: Dword,
    /// `dword_755934`: 1 once the welcome page was shown.
    pub welcomed: Dword,
    /// `g_serviceDue`: 1 when the service is due.
    pub service_due: Dword,
    /// `g_serviceMinutes`: The service time left.
    pub service_minutes: Dword,
    /// `dword_755940`: The service's work value (`menu::run_championship`).
    pub service_940: Dword,
    /// `dword_755944`: The first / service work value.
    pub service_944: Dword,
    /// `dword_755948`: The step of `menu::calc_cpu_times`.
    pub dword_755948: Dword,
    /// `dword_75594C`: The next service.
    pub service_next: Dword,
    /// `g_stagesDone`: The stages done in the event.
    pub stages_done: Dword,
    /// `g_eventType`: 3 time trial, 4 and 5 the championships (`menu::event_flow`).
    pub event_type: Dword,
    /// `index`: The event's stage in its section.
    pub index: Dword,
    /// `section`: The event's section.
    pub section: Dword,
    /// `dword_755960`: The first stage of the page shown.
    pub first_page_stage: Dword,
    /// `dword_755964`: Not worked out.
    pub dword_755964: Dword,
    /// `g_stagePages`: The stage pages.
    pub stage_pages: Dword,
    /// `g_stagePage`: The stage page shown.
    pub stage_page: Dword,
    /// `g_continuingSave`: 1 when a saved game goes on.
    pub continuing_save: Dword,
    /// Unreferenced bytes (IDA 0x755974 .. 0x755978).
    pub _gap_755974: Bytes<4>,
    /// `dword_755978`: 1 when the player is out of the event.
    pub event_out: Dword,
    /// `dword_75597C`: The count of the standings (`menu::number_all_a`).
    pub count_a: Dword,
    /// `dword_755980`: 1 when an event is over.
    pub event_done: Dword,
    /// `dword_755984`: The players in the event.
    pub player_count: Dword,
    /// `g_eventKind`: The event's kind (9 .. 0x18, `menu::run_event_races`).
    pub event_kind: Dword,
    /// `g_currentPlayer`: The player whose turn it is.
    pub current_player: Dword,
    /// `dword_755990`: The car shown (an entry of the carousel).
    pub shown_car: Dword,
    /// `dword_755994`: The slideshow's picture, 0 .. 99.
    pub picture: Dword,
    /// `dword_755998`: The car page shown.
    pub car_page: Dword,
    /// Unreferenced bytes (IDA 0x75599C .. 0x7559A4).
    pub _gap_75599c: Bytes<8>,
}

impl Event {
    /// First values the image's (`tests::event_state_starts_as_the_games`).
    pub const fn new() -> Self {
        Event {
            dword_7558c4: Dword::new(0x0),
            stage_group: Dword::new(0x0),
            stage_number: Dword::new(0x1),
            stage_sub: Dword::new(0x0),
            dword_7558d4: Dword::new(0x0),
            clears_escape_flag: Dword::new(0x0),
            sparks_mode: Dword::new(0x0),
            dword_7558e0: Dword::new(0x0),
            dword_7558e4: Dword::new(0x0),
            dword_7558e8: Dword::new(0x0),
            dword_7558ec: Dword::new(0x0),
            dword_7558f0: Dword::new(0x0),
            class: Dword::new(0x0),
            classes: Dword::new(0x4),
            dword_7558fc: Dword::new(0x0),
            dword_755900: Dword::new(0x0),
            dword_755904: Dword::new(0x0),
            dword_755908: Dword::new(0x0),
            dword_75590c: Dword::new(0x0),
            dword_755910: Dword::new(0x2),
            player_time: Dword::new(0x0),
            time_qualifies: Dword::new(0x0),
            dword_75591c: Dword::new(0x0),
            any_time_qualified: Dword::new(0x0),
            welcome_figure: Dword::new(0x3DC),
            championship_end: Dword::new(0x0),
            event_goes_on: Dword::new(0x1),
            dword_755930: Dword::new(0x0),
            welcomed: Dword::new(0x0),
            service_due: Dword::new(0x0),
            service_minutes: Dword::new(0x0),
            service_940: Dword::new(0x0),
            service_944: Dword::new(0x0),
            dword_755948: Dword::new(0x0),
            service_next: Dword::new(0x0),
            stages_done: Dword::new(0x0),
            event_type: Dword::new(0x1),
            index: Dword::new(0x0),
            section: Dword::new(0x0),
            first_page_stage: Dword::new(0x0),
            dword_755964: Dword::new(0x0),
            stage_pages: Dword::new(0x0),
            stage_page: Dword::new(0x0),
            continuing_save: Dword::new(0x0),
            _gap_755974: Bytes::new(with_runs::<4>(&[(0x0, &[0x05])])),
            event_out: Dword::new(0x0),
            count_a: Dword::new(0x0),
            event_done: Dword::new(0x0),
            player_count: Dword::new(0x1),
            event_kind: Dword::new(0x18),
            current_player: Dword::new(0x0),
            shown_car: Dword::new(0x0),
            picture: Dword::new(0x0),
            car_page: Dword::new(0x0),
            _gap_75599c: Bytes::new([0; 8]),
        }
    }
}

/// IDA 0x7558C4, the first byte of the piece.
pub const START: usize = 0x7558C4;
/// IDA 0x7559A4, the end of the piece.
pub const END: usize = 0x7559A4;

/// Where IDA 0x7558C4 is now (`moved::MOVED`'s home).
#[cfg(test)]
pub fn event_addr() -> u32 {
    crate::pointers::game_address(&raw const crate::menu::owner::MENUS.screens.event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn event_state_starts_as_the_games() {
        assert!(
            END - START <= size_of::<Event>()
                && size_of::<Event>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(event_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(Event, dword_7558c4), 0x7558C4);
        at(offset_of!(Event, stage_group), 0x7558C8);
        at(offset_of!(Event, stage_number), 0x7558CC);
        at(offset_of!(Event, stage_sub), 0x7558D0);
        at(offset_of!(Event, dword_7558d4), 0x7558D4);
        at(offset_of!(Event, clears_escape_flag), 0x7558D8);
        at(offset_of!(Event, sparks_mode), 0x7558DC);
        at(offset_of!(Event, dword_7558e0), 0x7558E0);
        at(offset_of!(Event, dword_7558e4), 0x7558E4);
        at(offset_of!(Event, dword_7558e8), 0x7558E8);
        at(offset_of!(Event, dword_7558ec), 0x7558EC);
        at(offset_of!(Event, dword_7558f0), 0x7558F0);
        at(offset_of!(Event, class), 0x7558F4);
        at(offset_of!(Event, classes), 0x7558F8);
        at(offset_of!(Event, dword_7558fc), 0x7558FC);
        at(offset_of!(Event, dword_755900), 0x755900);
        at(offset_of!(Event, dword_755904), 0x755904);
        at(offset_of!(Event, dword_755908), 0x755908);
        at(offset_of!(Event, dword_75590c), 0x75590C);
        at(offset_of!(Event, dword_755910), 0x755910);
        at(offset_of!(Event, player_time), 0x755914);
        at(offset_of!(Event, time_qualifies), 0x755918);
        at(offset_of!(Event, dword_75591c), 0x75591C);
        at(offset_of!(Event, any_time_qualified), 0x755920);
        at(offset_of!(Event, welcome_figure), 0x755924);
        at(offset_of!(Event, championship_end), 0x755928);
        at(offset_of!(Event, event_goes_on), 0x75592C);
        at(offset_of!(Event, dword_755930), 0x755930);
        at(offset_of!(Event, welcomed), 0x755934);
        at(offset_of!(Event, service_due), 0x755938);
        at(offset_of!(Event, service_minutes), 0x75593C);
        at(offset_of!(Event, service_940), 0x755940);
        at(offset_of!(Event, service_944), 0x755944);
        at(offset_of!(Event, dword_755948), 0x755948);
        at(offset_of!(Event, service_next), 0x75594C);
        at(offset_of!(Event, stages_done), 0x755950);
        at(offset_of!(Event, event_type), 0x755954);
        at(offset_of!(Event, index), 0x755958);
        at(offset_of!(Event, section), 0x75595C);
        at(offset_of!(Event, first_page_stage), 0x755960);
        at(offset_of!(Event, dword_755964), 0x755964);
        at(offset_of!(Event, stage_pages), 0x755968);
        at(offset_of!(Event, stage_page), 0x75596C);
        at(offset_of!(Event, continuing_save), 0x755970);
        at(offset_of!(Event, event_out), 0x755978);
        at(offset_of!(Event, count_a), 0x75597C);
        at(offset_of!(Event, event_done), 0x755980);
        at(offset_of!(Event, player_count), 0x755984);
        at(offset_of!(Event, event_kind), 0x755988);
        at(offset_of!(Event, current_player), 0x75598C);
        at(offset_of!(Event, shown_car), 0x755990);
        at(offset_of!(Event, picture), 0x755994);
        at(offset_of!(Event, car_page), 0x755998);
    }
}
