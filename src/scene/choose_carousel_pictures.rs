//! `scene::choose_carousel_pictures` (0x4901B9): on the menus whose carousel shows
//! pictures, the four entries that can be seen get theirs.
//!
//! In:  nothing
//! Out: nothing changed on other menus; else as `scene::update_carousel_pictures`
//!
//! Only with `g_carouselMenu` at 0xB .. 0xE: the entry before the front one
//! (`g_carouselFront`, going round `g_carouselCount`) and the three after it are the
//! four entries of `scene::update_carousel_pictures`.
//! One caller, `scene::draw_main_menu_objects`.
//! Self-test: 80 settings by lot, with records of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::scene::carousel_pictures::{slot_regions, update};
use crate::scene::textures::SLOTS;
use crate::scene::update_carousel_pictures::World;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::choose_carousel_pictures",
    rva(0x4901B9),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const MENU_RVA: usize = rva(0x75AA74); // g_carouselMenu
const COUNT_RVA: usize = rva(0x75AA78); // g_carouselCount
const FRONT_RVA: usize = rva(0x75AA7C); // g_carouselFront

// ---- Rust version ----

pub unsafe fn choose_carousel_pictures() -> Result<Left, Refused> {
    // Converted: `rc2k_core::scene::carousel::{shows_pictures, visible_entries}`.
    use rc2k_core::scene::carousel::{shows_pictures, visible_entries};
    unsafe {
        if !shows_pictures(game_read(MENU_RVA)) {
            return Ok(Left::nothing());
        }
        Ok(update(visible_entries(
            game_read(FRONT_RVA),
            game_read(COUNT_RVA),
        ))?)
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { choose_carousel_pictures() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &slot_regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = [at(MENU_RVA), at(COUNT_RVA), at(FRONT_RVA)];
    let saved = capture(&inputs);
    let world = World::new();
    let mut seed = 0x7E87_0005u32;
    let mut lot = |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) % below
    };
    for case in 0..80 {
        let menu = [0xAu32, 0xB, 0xC, 0xD, 0xE, 0xF, 0][lot(7) as usize];
        // Every entry reached is one of the world's six.
        let count = 1 + lot(6);
        let front = lot(count);
        let held = [
            world.picture(lot(6)),
            u32::MAX,
            world.picture(lot(6)),
            lot(0x50),
        ];
        let slot = [0x40u32, SLOTS - 32][lot(2) as usize];
        check_state(
            &mut t,
            &format!("case {case}: menu {menu:#x}, {count} entries, front {front}, held {held:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                world.set([0xAAAA_AAAA; 4], held, slot, 0x500);
                game_write(MENU_RVA, menu);
                game_write(COUNT_RVA, count);
                game_write(FRONT_RVA, front);
            },
            world.regions(),
            &rust,
        );
    }
    drop(world);
    restore(&inputs, &saved);
    t.finish()
}
