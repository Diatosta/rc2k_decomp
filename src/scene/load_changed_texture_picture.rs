//! `scene::load_changed_texture_picture` (0x42574B): a system texture gets the picture
//! it should have, if it has another.
//!
//! In:  ecx = the texture
//! Out: every register preserved
//!
//! Nothing if the texture wants no picture (`dword_5D8968[texture]` is -1 or 0) or
//! has the one it wants (`dword_5D9968[texture]`). Else `scene::load_texture_picture`
//! and the picture is noted as the one it has.
//! One caller, `scene::reload_system_textures`.
//! Refuses a texture past the tables.
//! Shadow mode runs the original alone and checks what the texture has afterwards.
//! Self-test: a redirected copy whose loading notes its registers.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::load_texture_picture::load_texture_picture_in_game;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::load_changed_texture_picture",
    rva(0x42574B),
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

pub const WANTED_PICTURES_RVA: usize = rva(0x5D8968); // dword_5D8968
pub const LOADED_PICTURES_RVA: usize = rva(0x5D9968); // dword_5D9968
pub const TEXTURES: u32 = 0x400;

// ---- Rust version ----

/// `load`: what stands for `scene::load_texture_picture`, given the picture and the
/// texture.
pub unsafe fn load_changed_texture_picture(
    texture: u32,
    load: &mut dyn FnMut(u32, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        if texture >= TEXTURES {
            return Err(BadInput("a texture past the pictures' tables").into());
        }
        let at = texture as usize * 4;
        let wanted: u32 = game_read(WANTED_PICTURES_RVA + at);
        if wanted == u32::MAX || wanted == 0 || wanted == game_read::<u32>(LOADED_PICTURES_RVA + at)
        {
            return Ok(());
        }
        load(wanted, texture)?;
        game_write(LOADED_PICTURES_RVA + at, wanted);
        Ok(())
    }
}

pub unsafe fn load_changed_texture_picture_in_game(texture: u32) -> Result<(), Refused> {
    unsafe {
        load_changed_texture_picture(texture, &mut |picture, texture| {
            load_texture_picture_in_game(picture, texture)
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        let texture = r.ecx;
        let wanted = (texture < TEXTURES)
            .then(|| unsafe { game_read::<u32>(WANTED_PICTURES_RVA + texture as usize * 4) });
        unsafe { call_original(r) };
        if let Some(wanted) = wanted.filter(|&wanted| wanted != u32::MAX && wanted != 0) {
            let has: u32 = unsafe { game_read(LOADED_PICTURES_RVA + texture as usize * 4) };
            if has != wanted {
                HOOK.mismatch(|| {
                    format!("texture {texture:#x} wanted {wanted:#x}: it has {has:#x} afterwards")
                });
            }
        }
        return;
    }
    if let Err(e) = unsafe { load_changed_texture_picture_in_game(r.ecx) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x2D;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8B, 0x04, 0x8D, 0x68, 0x89, 0x5D, 0x00];
const CALLS: [usize; 1] = [0x1E];
const TEXTURE: u32 = 0x3F9;

/// Each loading: eax, ebx, and what the texture has by then.
static CALLED: Mutex<Vec<[u32; 3]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    let has: u32 = unsafe { game_read(LOADED_PICTURES_RVA + TEXTURE as usize * 4) };
    CALLED.lock().unwrap().push([r.eax, r.ebx, has]);
    r.eax = 0x7777;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x42574B)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x42578E)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let at_texture = TEXTURE as usize * 4;
    let regions = [
        at(WANTED_PICTURES_RVA + at_texture),
        at(LOADED_PICTURES_RVA + at_texture),
    ];
    let saved = capture(&regions);
    for (wanted, has) in [
        (u32::MAX, 5u32),
        (0, 5),
        (0, 0),
        (7, 7),
        (7, 8),
        (0x7D1, u32::MAX),
        (0xFFFF_FFFE, 0),
    ] {
        let run = |rust: bool| unsafe {
            game_write(WANTED_PICTURES_RVA + at_texture, wanted);
            game_write(LOADED_PICTURES_RVA + at_texture, has);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs {
                ecx: TEXTURE,
                ..Regs::SENTINEL
            };
            let kept = if rust {
                load_changed_texture_picture(TEXTURE, &mut |picture, texture| {
                    act(
                        0,
                        &mut Regs {
                            eax: picture,
                            ebx: texture,
                            ..before
                        },
                    );
                    Ok(())
                })
                .is_ok()
            } else {
                let mut r = before;
                call_game(&mut r, copy.addr());
                r.preserved_except(&before, 0)
            };
            (kept, capture(&regions), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!("wanted {wanted:#x}, has {has:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
