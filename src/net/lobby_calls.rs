//! The lobby's calls in the game, by place (the callee's IDA address): the Rust versions
//! (a refusal is the caller's); the screens' frames, the screen modes, the window's
//! messages, the car and stage screens, the music and the race sounds' set-up through the
//! game (their hooks); DirectPlay's `Send` and the received messages' lock as `net::game_part`
//! has them.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::*;
use crate::text::draw_text::{GameCanvas, Outcome};
use crate::text::font::Font;

/// The registers `draw_text` leaves.
fn drawn(r: &mut Regs, out: &Outcome) {
    (r.eax, r.ebx, r.ecx, r.edx) = (0, out.text as u32, out.x as u32, out.y as u32);
    r.set_carry(out.too_wide);
    r.set_zero(!out.too_wide);
}

/// Writes bytes at the cursor in ebx and moves it on.
fn emit(r: &mut Regs, bytes: &[u8]) {
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), r.ebx as *mut u8, bytes.len()) };
    r.ebx = r.ebx.wrapping_add(bytes.len() as u32);
}

pub fn call(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_hex8::log_hex8, log_int::log_int, log_write::log_write};
    unsafe {
        match place {
            // The log: each stub's level in edx, as the game's stubs leave it.
            LOG_L1 | LOG_L1_NL | LOG_L2 | LOG_L2_NL => {
                let (newline, level) = match place {
                    LOG_L1 => (false, 1),
                    LOG_L1_NL => (true, 1),
                    LOG_L2 => (false, 2),
                    _ => (true, 2),
                };
                r.edx = (level as u32 - 1) << 8 | newline as u32;
                log_write(r.eax as *const u8, newline, level);
            }
            LOG_INT_L1_NL => {
                r.edx = 1;
                log_int(r.eax as i32, true, 1);
            }
            LOG_INT_L2_NL => {
                r.edx = 0x101;
                log_int(r.eax as i32, true, 2);
            }
            LOG_HEX_L2 | LOG_HEX_L2_NL => {
                let newline = place == LOG_HEX_L2_NL;
                r.edx = 0x100 | newline as u32;
                log_hex8(r.eax, newline, 2);
            }
            LOG_NEWLINE_L2 => log_write(game(0x4E00B7) as *const u8, true, 2),
            SAME_GUID => {
                let same = crate::system::same_guid::same_guid(r.eax, r.ebx);
                r.set_zero(same);
            }
            COPY_PRINTABLE => crate::strings::copy_printable::copy_printable(
                r.eax as *const u8,
                r.ebx as *mut u8,
                r.ecx,
            )?,
            ZERO_MEMORY => {
                crate::system::zero_memory::zero_memory(r.ebx as *mut u8, r.ecx)?;
                r.set_carry(false);
                r.set_zero(r.ecx & 3 == 0);
            }
            NEXT_RANDOM => {
                r.eax = crate::math::next_random::next_random();
                r.set_zero(r.eax == 0);
            }
            APPEND_TIME => {
                let time = crate::strings::append_time::append_time(
                    r.eax,
                    crate::strings::append_time::show_hours(),
                );
                emit(r, &time);
            }
            SAME_TEXT => {
                let order = crate::strings::same_text::compare_texts(
                    r.eax as *const u8,
                    r.ebx as *const u8,
                );
                r.set_zero(order == std::cmp::Ordering::Equal);
                r.set_carry(order == std::cmp::Ordering::Less);
            }
            STR_APPEND => {
                r.ebx = crate::strings::str_append::str_append(r.eax as *const u8, r.ebx as *mut u8)
                    as u32
            }
            APPEND_STRING => {
                let mut out = r.ebx as *mut u8;
                r.eax = crate::strings::append_string_by_id::append_string_by_id(
                    r.eax,
                    &mut |from, n| {
                        std::ptr::copy_nonoverlapping(from, out, n);
                        out = out.add(n);
                    },
                );
                r.ebx = out as u32;
            }
            FORMAT_U32 => {
                let text = crate::strings::format_u32::format_u32(r.eax, r.ecx as u8);
                emit(r, &text);
            }
            TEXT_LENGTH => {
                r.ecx = crate::strings::text_length::text_length(r.ebx as *const u8)?;
                r.set_zero(true);
                r.set_carry(false);
            }
            DRAW_KEEP => {
                let font = r.eax as *mut Font;
                let mut copy = font.read_unaligned();
                let too_wide = crate::text::draw_text_keep_regs::draw_text_keep_regs(
                    &mut copy,
                    r.ebx as *const u8,
                    r.ecx as i32,
                    r.edx as i32,
                    &mut GameCanvas,
                )?;
                if copy != font.read_unaligned() {
                    font.write_unaligned(copy);
                }
                r.set_carry(too_wide);
                r.set_zero(!too_wide);
            }
            MEASURE_TEXT => {
                let m = crate::text::measure_text::measure_text_at(
                    r.eax as *mut Font,
                    r.ebx as *const u8,
                    r.ecx as i32,
                    &mut GameCanvas,
                )?;
                (r.eax, r.ebx, r.ecx, r.edx) = (0, m.lines, m.width, m.outcome.y as u32);
                r.set_carry(m.outcome.too_wide);
                r.set_zero(!m.outcome.too_wide);
            }
            MEASURE_STRING => {
                let m = crate::text::measure_string_by_id::measure_string_by_id_at(
                    r.eax,
                    r.ebx as *mut Font,
                    r.ecx as i32,
                    &mut GameCanvas,
                )?;
                (r.eax, r.ebx, r.ecx, r.edx) = (0, m.lines, m.width, m.outcome.y as u32);
                r.set_carry(m.outcome.too_wide);
                r.set_zero(!m.outcome.too_wide);
            }
            DRAW_STRING => {
                let out = crate::text::draw_string_by_id::draw_string_by_id_at(
                    r.eax,
                    r.ebx as *mut Font,
                    r.ecx as i32,
                    r.edx as i32,
                    &mut GameCanvas,
                )?;
                drawn(r, &out);
            }
            DRAW_TEXT => {
                let out = crate::text::draw_text::draw_text_at(
                    r.eax as *mut Font,
                    r.ebx as *const u8,
                    r.ecx as i32,
                    r.edx as i32,
                    &mut GameCanvas,
                )?;
                drawn(r, &out);
            }
            DRAW_TIME => {
                *r = crate::text::draw_time_text::draw_time_text(r)?;
            }
            ALLOCATE => {
                r.eax = crate::memory::allocate::allocate(r.ebx as *mut _, r.eax)? as u32;
                r.set_carry(false);
            }
            MAYBE_LEXICON => crate::strings::maybe_compile_lexicon::maybe_compile_lexicon()?,
            SET_RACE_MODE => crate::race::race_shell::set_race_screen_mode(
                &{ *r },
                &mut crate::race::race_shell::game_screen_mode,
            ),
            RESTORE_WINDOW => crate::system::restore_window::restore_window(),
            PICTURE_AS_SURFACE => {
                let _ = r;
                crate::gfx::picture_as_surface::picture_as_surface()?;
            }
            COPY_PICTURE => {
                let _ = r;
                crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
                    &mut crate::gfx::copy_picture_to_surface::GameSurface,
                )?;
            }
            CONVERT_PICTURE => {
                let _ = r;
                crate::gfx::convert_picture::convert_picture_now()?;
            }
            LOAD_PICTURE => {
                let read = crate::gfx::load_picture::load_picture(r.eax as *mut u8)?;
                r.set_carry(!read);
            }
            DRAW_HLINE => {
                crate::gfx::rect::draw_hline(r.ecx as i32, r.edx as i32, r.ebx as i32, r.eax, r.esi)
            }
            BLIT => {
                let exit = crate::gfx::blit_glyph::blit_glyph_on_surface(
                    r.eax,
                    r.esi as *const u8,
                    r.ecx as i32,
                    r.edx as i32,
                    r.ebx as *const u32,
                )?;
                (r.eax, r.ebx, r.ecx, r.edx) = (exit.eax, exit.ebx, exit.ecx, exit.edx);
            }
            SCREENSHOT => crate::gfx::take_screenshot::take_screenshot()?,
            KEY_TO_CHAR => {
                r.eax = crate::input::key_to_char::key_to_char(r.eax)?;
                r.set_carry(r.eax as u8 == 0);
                r.set_zero(r.eax as u8 == 1);
            }
            TAKE_KEY => crate::race::viewer_load_stage::take_key(r),
            STOP_MUSIC => crate::sound::stop_music::stop_music_in_game(r),
            SLEEP => crate::system::sleep::sleep(r.eax),
            TICK => r.eax = crate::system::tick_count::tick_count(),
            PARSE_NUMBER => r.eax = crate::settings::parse_number::parse_number(r.eax as *const u8),
            LIST_TAKE_KEY => *r = crate::menu::list_take_key::list_take_key(r)?,
            CLEAR_RACE_SOUNDS => {
                use crate::race::sound_tables;
                *r = sound_tables::clear_race_sound_objects(r)?;
            }
            SET_UP_RACE_SOUNDS => {
                use crate::race::sound_tables;
                sound_tables::set_up_race_sound_objects(r, &sound_tables::ObjectVars::game())?;
            }
            SET_UP_ENGINE => crate::scene::set_up_engine::set_up_engine_in_game(r),
            FRONT_END_SCENE => {
                crate::scene::set_up_front_end_scene::set_up_front_end_scene_in_game(r)?
            }
            RENDER3D_TIDY => crate::race::race_end::menu_part(1, r)?,
            OBJECTS_TIDY => crate::race::race_end::menu_part(2, r)?,
            RELEASE_TEXTURES => crate::race::race_end::menu_part(3, r)?,
            NULLSUB_6 => {}
            LOAD_STAGE => {
                *r = crate::race::stage_loading::load_stage(
                    r,
                    a(0x4E00A0), // g_logIndent
                    &mut crate::race::stage_loading::game_part,
                )?;
                r.set_carry(false);
            }
            TIDY_EVERYTHING => {
                use crate::race::{loadlevel::LoadVars, race_end};
                *r = race_end::tidy_everything(r, &LoadVars::game(), &mut race_end::tidy_part)?;
            }
            PLAY_RACE => crate::drive::play_race::play_race(r)?.apply(r),
            ARCADE_CARS => {
                let g = crate::race::race_cars::set_up_arcade_cars(
                    r,
                    a(0x4E00A0), // g_logIndent
                    &mut crate::race::race_cars::cars_part,
                )?;
                *r = Regs {
                    eflags: r.eflags,
                    ..g
                };
            }
            TAKE_SHOWN_CAR => crate::menu::take_shown_car::take_shown_car(r.esi)?,
            // The pieces of code the lobby posts to the receiving thread are posted as their
            // Rust versions (DirectPlay from Rust); `net::post_call` itself as it is.
            POST_CALL => {
                use crate::net::{find_sessions, host_session, join_session};
                let posted: extern "C" fn() -> u32 = match r.eax {
                    x if x == game(find_sessions::POSTED_CODE) => {
                        find_sessions::posted_enum_sessions
                    }
                    x if x == game(host_session::POSTED_CODE) => host_session::posted_open_host,
                    x if x == game(join_session::POSTED_CODE) => join_session::posted_open_join,
                    _ => {
                        return Err(crate::errors::BadInput("a posted call not known").into());
                    }
                };
                let mut c = Regs {
                    eax: posted as *const () as u32,
                    ..*r
                };
                c = crate::net::post_call::post_call(&c, &mut call)?;
                *r = c;
            }
            CO_CREATE => {
                r.eax = crate::net::dplay::create_instance(r.eax, r.ebx, r.ecx) as u32;
            }
            // DirectPlay's `Send` and the received messages' lock and stop: as `net` has
            // them.
            SEND | LOCK_RECEIVED | UNLOCK_RECEIVED | COPY_RECEIVED | STOP | PUMP => {
                crate::net::game_part(place, r)?
            }
            // The platform's (frames, screen modes, the front surface), and long runners
            // that call back into the game (the car and stage screens, the music, the race
            // sounds' set-up, the loading picture, the start states): through their hooks.
            LOCK | UNLOCK | SHOW_FRAME | RUN_FRAME | SET_MENU_MODE | CLEAR_FRONT | SELECT_CAR
            | SELECT_STAGE | TUNE_3 | TUNE_4 | APPLY_SOUND | RACE_SOUNDS | LOADING_PICTURE
            | START_STATES | START_FRONT_END => in_game(place, r),
            _ => return crate::net::lobby_hooks::call(place, r),
        }
    }
    Ok(())
}
