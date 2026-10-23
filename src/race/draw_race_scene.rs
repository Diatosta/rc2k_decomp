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
//! In:  nothing
//! Out: the registers as the last call leaves them; no flags read (`race_frame` goes on)
//!
//! Caller: `drive::race_frame`. It begins with a call: a private copy (its 80 calls
//! relocated) and a manual patch. Its calls through `part` by place (`CALLEES`); in the game
//! the Rust versions of the sound tick, the mud's frame and the three view passes, the
//! others through the game (`sub_455D38`, the platform's blit, section 15; the rest their
//! hooks). Refuses (before anything) a null view block, and a wanted texture's slot past
//! `g_slotWantedTextures`' 0x400. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose calls go to stand-ins, against the Rust version, on
//! view blocks of the test's: each layout, the switches on and off; the calls, their
//! registers and the variables compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 32] = [
    0x4560EA, // 0 drive::choose_shown_cars
    0x421BA9, // 1 sound::race_sound_tick
    0x44E8FB, // 2 scene::set_filters
    0x44EB72, // 3 scene::set_render_state
    0x450381, // 4 scene::restart_clock
    0x455ECB, // 5 race::full_view_window
    0x460D5C, // 6 race::ready_car_parts
    0x45B4E6, // 7 race::scroll_maps
    0x45B200, // 8 race::forget_view_textures
    0x44A054, // 9 race::mud_frame
    0x4256D8, // 10 scene::assign_kind_textures
    0x42570D, // 11 scene::map_texture_places
    0x425778, // 12 scene::load_slot_textures
    0x45AB36, // 13 race::set_up_panel
    0x45422F, // 14 race::load_car_a
    0x45550E, // 15 race::fetch_view_value
    0x4557AB, // 16 race::draw_view_scene
    0x455D38, // 17 sub_455D38: the platform's blit
    0x4559A6, // 18 race::draw_view_overlays
    0x454806, // 19 race::draw_mirror_view
    0x44E47B, // 20 scene::set_view_window
    0x454241, // 21 race::load_car_b
    0x460C7A, // 22 race::load_first_car
    0x45FDCF, // 23 race::flush_triangle_batch
    0x45FEFE, // 24 race::flush_texture_batch
    0x45C605, // 25 race::find_sound_spots
    0x45C7B9, // 26 race::share_sound_spots
    0x45D1EA, // 27 race::wake_part_sounds
    0x45A893, // 28 hud::overlay_pass
    0x4585F3, // 29 race::lay_skidmarks_if_on
    0x45B3B9, // 30 race::update_name_plates
    0x445047, // 31 drive::tyre_change_screen
];

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
const SCREEN_RVA: [usize; 2] = [rva(0x4F494C), rva(0x4F4950)]; // g_screenWidth, g_screenHeight
const WINDOW_RVA: [usize; 5] = [
    rva(0x62408C), // width
    rva(0x623FB0), // x
    rva(0x624090), // height
    rva(0x623FB4), // y
    rva(0x624080), // 0x100
];
const CREDITS_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const TYRE_SCREEN_OFF_RVA: usize = rva(0x5C5CE0); // dword_5C5CE0

/// A call by place (`CALLEES`).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

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

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_race_scene(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let blocks = view_blocks()?;
    for (k, extra) in EXTRA_TEXTURES.into_iter().enumerate() {
        if g(extra) != u32::MAX && g(FIRST_SLOT_DC_RVA + 4 * k) >= SLOTS {
            return Err(BadInput("a wanted texture's slot past g_slotWantedTextures").into());
        }
    }
    let mut r = *entry;
    let tick = |r: &mut Regs, n: u32, part: &mut Part| {
        r.eax = n;
        part(1, r)
    };
    let mut part = part;
    part(0, &mut r)?;
    unsafe {
        game_write(CLEARED_RVA, [0u32; 0x20]);
    }
    r.edi = (game_addr(CLEARED_RVA) as u32).wrapping_add(0x80);
    r.ecx = 0;
    for k in 0..2 {
        r.eax = at_block(blocks[k], 0x220);
        p(VIEW_FLAGS[k], r.eax);
    }
    for k in 0..2 {
        if g(VIEW_ON[k]) == 0 {
            p(VIEW_FLAGS[k], 0);
        }
    }
    for a in ZEROED {
        p(a, 0);
    }
    r.eax = (g(BITS_A.0) & 2) >> 1;
    p(BITS_A.1, r.eax);
    r.eax = g(BITS_B.0) & 1;
    p(BITS_B.1, r.eax);
    r.eax = g(VIEW_FLAGS[0]) & 0x40;
    p(FIRST_BIT_6_RVA, r.eax);
    p(BLIT_SWITCH_RVA, 0);
    r.eax = g(BLIT_ON_RVA) | g(BLIT_ON_RVA + 4);
    if r.eax != 0 {
        r.eax = at_block(blocks[0], 0x6F0);
        p(BLIT_SWITCH_RVA, r.eax);
    }
    p(LIGHTING_RVA, 1);
    tick(&mut r, 0xA, &mut part)?;
    r.eax = g(SHINE_FROM_RVA);
    p(SHINE_RVA, r.eax);
    {
        // pusha; pushf .. popf; popa
        let kept = r;
        part(2, &mut r)?;
        for (value, state) in [(4, 0x15), (1, 0x1C), (5, 0x13), (6, 0x14), (1, 0x1B)] {
            r.eax = value;
            r.ebx = state;
            part(3, &mut r)?;
        }
        r = kept;
    }
    tick(&mut r, 0xB, &mut part)?;
    p(FRAMES_RVA, g(FRAMES_RVA).wrapping_add(1));
    part(4, &mut r)?;
    p(CLOCK_RVA, r.eax);
    for place in [5, 6, 7] {
        part(place, &mut r)?;
    }
    p(TEXTURE_CHANGES_RVA, 0);
    part(8, &mut r)?;
    part(9, &mut r)?;
    p(KIND_TEXTURES_DONE_RVA, 0);
    part(10, &mut r)?;
    for (k, extra) in EXTRA_TEXTURES.into_iter().enumerate() {
        r.eax = g(extra);
        if r.eax != u32::MAX {
            r.eax = r.eax.wrapping_add(g(FIRST_TEXTURE_DC_RVA));
            r.ebx = g(FIRST_SLOT_DC_RVA + 4 * k);
            p(SLOT_WANTED_RVA.wrapping_add(4 * r.ebx as usize), r.eax);
        }
    }
    part(11, &mut r)?;
    part(12, &mut r)?;
    tick(&mut r, 0xC, &mut part)?;
    if g(NO_PANEL_RVA) == 0 {
        part(13, &mut r)?;
    }
    if g(ONE_VIEW_RVA) == 1 {
        p(DRAWN_VIEW_RVA, 0);
        part(14, &mut r)?;
        part(15, &mut r)?;
        part(16, &mut r)?;
        tick(&mut r, 0xD, &mut part)?;
        part(17, &mut r)?;
        tick(&mut r, 0xE, &mut part)?;
        part(18, &mut r)?;
        tick(&mut r, 0xF, &mut part)?;
        if g(VIEW_FLAGS[0]) & 8 != 0 {
            part(14, &mut r)?;
            part(19, &mut r)?;
        }
    } else {
        let side_by_side = g(SIDE_BY_SIDE_RVA) == 1;
        for view in 0..2u32 {
            p(DRAWN_VIEW_RVA, view);
            let (w, h) = (g(SCREEN_RVA[0]), g(SCREEN_RVA[1]));
            // (width, x, height, y): the eax the original has for each store.
            if side_by_side {
                r.eax = w >> 1;
                p(WINDOW_RVA[0], r.eax);
                p(WINDOW_RVA[1], if view == 0 { 0 } else { r.eax });
                r.eax = h;
                p(WINDOW_RVA[2], r.eax);
                p(WINDOW_RVA[3], 0);
            } else {
                r.eax = w;
                p(WINDOW_RVA[0], r.eax);
                p(WINDOW_RVA[1], 0);
                r.eax = h >> 1;
                p(WINDOW_RVA[2], r.eax);
                p(WINDOW_RVA[3], if view == 0 { 0 } else { r.eax });
            }
            p(WINDOW_RVA[4], 0x100);
            part(20, &mut r)?;
            let car = if view == 0 { 14 } else { 21 };
            part(car, &mut r)?;
            part(15, &mut r)?;
            part(16, &mut r)?;
            tick(&mut r, 0xD, &mut part)?;
            part(18, &mut r)?;
            tick(&mut r, 0xF, &mut part)?;
            if g(VIEW_FLAGS[view as usize]) & 8 != 0 {
                part(car, &mut r)?;
                part(19, &mut r)?;
            }
        }
    }
    for place in [22, 23, 24, 25, 26, 27] {
        part(place, &mut r)?;
    }
    tick(&mut r, 0x10, &mut part)?;
    for state in [0x11, 0x12] {
        r.eax = 2;
        r.ebx = state;
        part(3, &mut r)?;
    }
    if g(CREDITS_RVA) == 0 && g(NO_PANEL_RVA) == 0 {
        part(28, &mut r)?;
    }
    tick(&mut r, 0x11, &mut part)?;
    part(29, &mut r)?;
    part(30, &mut r)?;
    if g(TYRE_SCREEN_OFF_RVA) != 2 {
        part(31, &mut r)?;
    }
    Ok(r)
}

/// In the game: the Rust versions of the sound tick, the mud's frame and the three view
/// passes; the others through the game (their hooks; `sub_455D38` the platform's).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => {
                return crate::sound::race_sound_tick::race_sound_tick_in(
                    r,
                    &mut crate::sound::turn_parts::game_part,
                );
            }
            9 => *r = crate::race::mud_frame::mud_frame(r, &mut crate::race::mud_frame::game_part)?,
            16 => crate::race::draw_view_scene::draw_view_scene_in_game(r)?,
            18 => {
                *r = crate::race::draw_view_overlays::draw_view_overlays(
                    r,
                    &mut crate::race::draw_view_overlays::game_part,
                )?
            }
            19 => {
                *r = crate::race::draw_mirror_view::draw_mirror_view(
                    r,
                    &mut crate::race::draw_mirror_view::game_part,
                )?
            }
            _ => crate::calls::call(rva(CALLEES[place as usize]), r),
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_race_scene(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
