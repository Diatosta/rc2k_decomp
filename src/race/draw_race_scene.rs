//! `race::draw_race_scene` (0x45429B): the race frame's drawing, between the scene's begin
//! and end (`drive::race_frame` part 3). In turn:
//! - the cars shown chosen (`drive::choose_shown_cars`), 0x20 dwords at `unk_63DBB8` 0; the
//!   two view blocks' +0x220 into `dword_6D14F6` / `dword_6D14FA` (each 0 while its
//!   `dword_63DB74` / `dword_63DB78` is 0); the batches' counts, `dword_639960` / `64`,
//!   `g_carsDrawn` and `g_carsCulled` 0; bit 1 of `dword_609354` into `dword_62DAD0`, bit 0
//!   of `dword_609350` into `dword_62DACC`, bit 6 of the first view's flags into
//!   `dword_6D1502`; `dword_6415E0` the first view block's +0x6F0 when `qword_609344` is not
//!   0, else 0; `dword_5CDA8C` 1;
//! - the sound tick (eax 0xA), `g_shine` from `dword_609324`; inside `pusha` / `pushf` the
//!   filters and five render states (0x15 = 4, 0x1C = 1, 0x13 = 5, 0x14 = 6, 0x1B = 1);
//! - the sound tick (0xB), `dword_63967C` one more, the clock restarted into
//!   `dword_63DC38`, the full view's window, the cars' parts readied, the maps scrolled;
//!   `dword_6D1508` 0, the views' textures forgotten, the mud's frame; `dword_6415F8` 0, the
//!   kinds' textures assigned, the two textures of `unk_6D1482` / `unk_6D1492` (unless -1)
//!   wanted in kind 0xDC's first slot and the next; the places mapped, the slots' textures
//!   loaded; the sound tick (0xC); the panel set up unless `dword_64BB18`;
//! - the views: one (`dword_63DB98` 1: car A's, the scene, ticks 0xD and 0xE around
//!   `sub_455D38`, the overlays, tick 0xF, the mirror if the view's flags have bit 3), or
//!   two halves (`dword_6238A4` 1 side by side, else one above the other), each with its
//!   window (`dword_62408C` .. `dword_624080`, `scene::set_view_window`), its car, the
//!   scene, tick 0xD, the overlays, tick 0xF and its mirror;
//! - the first car loaded, both batches flushed, the sound spots found and shared, the
//!   parts' sounds woken, the sound tick (0x10), render states 0x11 and 0x12 = 2; the HUD's
//!   pass unless `dword_71C3E0` or `dword_64BB18`; the sound tick (0x11), the skidmarks laid,
//!   the name plates; the tyre change screen unless `dword_5C5CE0` is 2.
//!
//! In:  nothing (the original reads no register at entry; nor do its steps read one it
//!      gives them: each sets what it uses)
//! Out: nothing its callers (`drive::race_frame`, `race::show_replay_still`) read
//!
//! Refuses (before anything) a null view block, and a wanted texture's slot past
//! `g_slotWantedTextures`' 0x400; then what its steps refuse. Hook removed 2026-10-23
//! (conversion phase 4); its steps called directly since then (conversion phase 3, no
//! longer by address; the platform's blit, `sub_455D38`, is `d3d::copy_screen_to_texture`).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::session::SCREEN;

const VIEW_BLOCKS_RVA: usize = rva(0x63DBB0); // g_viewBlocks
const CLEARED_RVA: usize = rva(0x63DBB8); // unk_63DBB8, 0x20 dwords
const VIEW_FLAGS: [usize; 2] = [rva(0x6D14F6), rva(0x6D14FA)]; // dword_6D14F6, dword_6D14FA
const VIEW_ON: [usize; 2] = [rva(0x63DB74), rva(0x63DB78)]; // dword_63DB74, dword_63DB78
const ZEROED: [usize; 8] = [
    rva(0x639964),
    rva(0x639960),
    rva(0x6E44F8),
    rva(0x6E44F4),
    rva(0x6EF7C0),
    rva(0x6EF7BC),
    rva(0x641594), // g_carsDrawn
    rva(0x641598), // g_carsCulled
];
const BITS_A: (usize, usize) = (rva(0x609354), rva(0x62DAD0));
const BITS_B: (usize, usize) = (rva(0x609350), rva(0x62DACC));
const FIRST_BIT_6_RVA: usize = rva(0x6D1502); // dword_6D1502
const BLIT_SWITCH_RVA: usize = rva(0x6415E0); // dword_6415E0
const BLIT_ON_RVA: usize = rva(0x609344); // qword_609344
const LIGHTING_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const SHINE_FROM_RVA: usize = rva(0x609324); // dword_609324
const SHINE_RVA: usize = rva(0x5CCE74); // g_shine
const FRAMES_RVA: usize = rva(0x63967C); // dword_63967C
const CLOCK_RVA: usize = rva(0x63DC38); // dword_63DC38
const TEXTURE_CHANGES_RVA: usize = rva(0x6D1508); // dword_6D1508
const KIND_TEXTURES_DONE_RVA: usize = rva(0x6415F8); // dword_6415F8
const EXTRA_TEXTURES: [usize; 2] = [rva(0x6D1482), rva(0x6D1492)]; // unk_6D1482, unk_6D1492
const FIRST_TEXTURE_DC_RVA: usize = rva(0x5D0E58); // g_kindFirstTextures[0xDC]
const FIRST_SLOT_DC_RVA: usize = rva(0x5D1E58); // g_kindFirstSlots[0xDC], [0xDD] next
const SLOT_WANTED_RVA: usize = rva(0x5DA968); // g_slotWantedTextures
const SLOTS: u32 = 0x400;
const NO_PANEL_RVA: usize = rva(0x64BB18); // dword_64BB18
const ONE_VIEW_RVA: usize = rva(0x63DB98); // dword_63DB98
const SIDE_BY_SIDE_RVA: usize = rva(0x6238A4); // dword_6238A4
const DRAWN_VIEW_RVA: usize = rva(0x63DB9C); // g_drawnView
const WINDOW_RVA: [usize; 5] = [
    rva(0x62408C), // width
    rva(0x623FB0), // x
    rva(0x624090), // height
    rva(0x623FB4), // y
    rva(0x624080), // 0x100
];
const CREDITS_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const TYRE_SCREEN_OFF_RVA: usize = rva(0x5C5CE0); // dword_5C5CE0

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}

/// The two view blocks, or the refusal.
fn view_blocks() -> Result<[u32; 2], Refused> {
    let blocks: [u32; 2] = unsafe { game_read(VIEW_BLOCKS_RVA) };
    if blocks.contains(&0) {
        return Err(BadInput("a null view block").into());
    }
    Ok(blocks)
}

fn at_block(block: u32, offset: u32) -> u32 {
    unsafe { ((block + offset) as *const u32).read_unaligned() }
}

/// One view drawn: `car` (0 or 1: `race::load_car_a` / `_b`), its value fetched, its scene,
/// the sound tick (0xD), the platform's blit (the full view only), the tick (0xE, the full
/// view only), the overlays, the tick (0xF), and with the view's flags' bit 3 its mirror.
unsafe fn draw_view(car: usize, flags: usize, blit: bool) -> Result<(), Refused> {
    use crate::race::frame::load_view_car;
    use crate::sound::race_sound_tick::race_sound_tick;
    unsafe {
        load_view_car(car)?;
        crate::race::frame_draw::fetch_view_value()?;
        crate::race::draw_view_scene::draw_view_scene()?;
        race_sound_tick()?; // 0xD
        if blit {
            crate::d3d::copy_screen_to_texture::copy_screen_to_texture();
            race_sound_tick()?; // 0xE
        }
        crate::race::draw_view_overlays::draw_view_overlays()?;
        race_sound_tick()?; // 0xF
        if g(VIEW_FLAGS[flags]) & 8 != 0 {
            load_view_car(car)?;
            crate::race::draw_mirror_view::draw_mirror_view()?;
        }
        Ok(())
    }
}

/// The Rust version (see the header).
pub unsafe fn draw_race_scene() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    use crate::sound::race_sound_tick::race_sound_tick;
    let blocks = view_blocks()?;
    for (k, extra) in EXTRA_TEXTURES.into_iter().enumerate() {
        if g(extra) != u32::MAX && g(FIRST_SLOT_DC_RVA + 4 * k) >= SLOTS {
            return Err(BadInput("a wanted texture's slot past g_slotWantedTextures").into());
        }
    }
    unsafe {
        crate::drive::choose_shown_cars::choose_shown_cars()?;
        game_write(CLEARED_RVA, [0u32; 0x20]);
        for k in 0..2 {
            p(VIEW_FLAGS[k], at_block(blocks[k], 0x220));
        }
        for k in 0..2 {
            if g(VIEW_ON[k]) == 0 {
                p(VIEW_FLAGS[k], 0);
            }
        }
        for a in ZEROED {
            p(a, 0);
        }
        p(BITS_A.1, (g(BITS_A.0) & 2) >> 1);
        p(BITS_B.1, g(BITS_B.0) & 1);
        p(FIRST_BIT_6_RVA, g(VIEW_FLAGS[0]) & 0x40);
        p(BLIT_SWITCH_RVA, 0);
        if g(BLIT_ON_RVA) | g(BLIT_ON_RVA + 4) != 0 {
            p(BLIT_SWITCH_RVA, at_block(blocks[0], 0x6F0));
        }
        p(LIGHTING_RVA, 1);
        race_sound_tick()?; // 0xA
        p(SHINE_RVA, g(SHINE_FROM_RVA));
        // pusha; pushf .. popf; popa
        crate::scene::set_filters::set_filters();
        for (value, state) in [(4, 0x15), (1, 0x1C), (5, 0x13), (6, 0x14), (1, 0x1B)] {
            set_render_state(value, state);
        }
        race_sound_tick()?; // 0xB
        p(FRAMES_RVA, g(FRAMES_RVA).wrapping_add(1));
        p(
            CLOCK_RVA,
            crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count()),
        );
        crate::race::frame_views::full_view_window(true);
        crate::race::car_parts::ready_car_parts()?;
        crate::race::map_scroll::scroll_maps()?;
        p(TEXTURE_CHANGES_RVA, 0);
        crate::race::frame_views::forget_view_textures()?;
        crate::race::mud_frame::mud_frame()?;
        p(KIND_TEXTURES_DONE_RVA, 0);
        crate::scene::assign_kind_textures::assign_kind_textures()?;
        for (k, extra) in EXTRA_TEXTURES.into_iter().enumerate() {
            let texture = g(extra);
            if texture != u32::MAX {
                let slot = g(FIRST_SLOT_DC_RVA + 4 * k);
                p(
                    SLOT_WANTED_RVA.wrapping_add(4 * slot as usize),
                    texture.wrapping_add(g(FIRST_TEXTURE_DC_RVA)),
                );
            }
        }
        crate::scene::map_texture_places::map_texture_places();
        crate::scene::load_slot_textures::load_slot_textures_in_game()?;
        race_sound_tick()?; // 0xC
        if g(NO_PANEL_RVA) == 0 {
            crate::race::frame_draw::set_up_panel()?;
        }
        if g(ONE_VIEW_RVA) == 1 {
            p(DRAWN_VIEW_RVA, 0);
            draw_view(0, 0, true)?;
        } else {
            let side_by_side = g(SIDE_BY_SIDE_RVA) == 1;
            for view in 0..2u32 {
                p(DRAWN_VIEW_RVA, view);
                let (w, h) = (SCREEN.width.get(), SCREEN.height.get());
                // (width, x, height, y).
                if side_by_side {
                    p(WINDOW_RVA[0], w >> 1);
                    p(WINDOW_RVA[1], if view == 0 { 0 } else { w >> 1 });
                    p(WINDOW_RVA[2], h);
                    p(WINDOW_RVA[3], 0);
                } else {
                    p(WINDOW_RVA[0], w);
                    p(WINDOW_RVA[1], 0);
                    p(WINDOW_RVA[2], h >> 1);
                    p(WINDOW_RVA[3], if view == 0 { 0 } else { h >> 1 });
                }
                p(WINDOW_RVA[4], 0x100);
                crate::scene::set_view_window::set_view_window_plain();
                draw_view(view as usize, view as usize, false)?;
            }
        }
        crate::race::frame_views::load_first_car()?;
        use crate::race::frame_flush::{PLAIN, TEXTURED, flush_batch};
        flush_batch(&PLAIN, false)?;
        flush_batch(&TEXTURED, true)?;
        crate::race::sound_spots::find_sound_spots()?;
        crate::race::frame_draw::share_sound_spots()?;
        crate::race::frame_draw::wake_part_sounds()?;
        race_sound_tick()?; // 0x10
        for state in [0x11, 0x12] {
            set_render_state(2, state);
        }
        if g(CREDITS_RVA) == 0 && g(NO_PANEL_RVA) == 0 {
            crate::hud::overlay_pass::overlay_pass()?;
        }
        race_sound_tick()?; // 0x11
        crate::race::skid_laying::lay_skidmarks_if_on()?;
        crate::race::name_plates::update_name_plates()?;
        if g(TYRE_SCREEN_OFF_RVA) != 2 {
            crate::drive::tyre_change_screen::tyre_change_screen()?;
        }
    }
    Ok(())
}
