//! The Direct3D the game's 3D code stands on, for the standalone program: textures of
//! our own, the scene's beginning and end, the depth buffer's clearing. Not ports, as
//! in `platform.rs`: each takes the place of a function of the game's Direct3D layer,
//! keeps what that function leaves for the game's own code, and does the rest itself.
//! No self-tests: there is no original to compare with.
//!
//! The textures. The game keeps two pools of records of 0x78 bytes, up to 0x140 each:
//! system textures (`unk_5B521C`, count `dword_5B5218`), into which pictures are put,
//! and video textures (`unk_5ABC18`, count `dword_5ABC14`), which are drawn with. A
//! record (`TextureRecord`): +0 its kind (1 system, 2 video), +4 width, +8 height, +0xC its pixel kind's
//! description, +0x10 levels, +0x14 a surface a level, +0x38 a texture a level, +0x5C
//! (system) the alpha from a second picture, +0x60 locked, +0x64 (video) the system
//! texture it holds, +0x68 its texture, +0x6C its handle, +0x70 a material. Here every
//! surface is an object of ours, a pointer to `VTABLE` and its number, whose `Lock`
//! hands out memory of ours: so `gfx::upload_texture_picture`, which locks through the
//! surface's own table, fills it unchanged.
//!
//! - `d3d::find_texture_formats` (0x419F5A, under Direct3D's start-up; called here
//!   when the screen mode is set): the four descriptions as the device would have
//!   found its formats, 555, 1555, 4444 and 8888, and the three kinds pointed at them
//!   the original's way.
//! - `gfx::allocate_textures` (0x4051ED), `gfx::release_all_textures` (0x4051D3).
//! - `scene::load_texture` (0x44E778): a system texture into a video texture, level
//!   for level (the original's `Blt`).
//! - `d3d::begin_scene` (0x41ABC7), `d3d::end_scene` (0x41AB8A): the layer's own
//!   states and its "in a scene" switch.
//! - `d3d::clear_viewport` (0x44E9D9): the depth buffer.

use crate::errors::FatalError;
use crate::framework::{
    GameHook, RawHook, Regs, call_game, game_addr, game_ptr, game_read, game_write, hook_asm, log,
    rva,
};
use crate::gfx::describe_format::describe_format;
use crate::gfx::texture_upload::{TextureFormat, TextureRecord};
use crate::logfile::log_write::log_write;
use crate::system::find_entry::find_entry;
use std::sync::atomic::AtomicUsize;
use std::sync::{Mutex, OnceLock};

// ---- game data ----

/// The four descriptions of 0x40 bytes: 555, 1555, 4444, 8888.
const DESCRIPTIONS_RVA: [usize; 4] = [rva(0x5ABAD4), rva(0x5ABB14), rva(0x5ABB54), rva(0x5ABB94)];
const KINDS_RVA: usize = rva(0x5ABAC0); // dword_5ABAC0: a description a pixel kind
const VIDEO_RECORDS_RVA: usize = rva(0x5ABC18); // unk_5ABC18
const SYSTEM_RECORDS_RVA: usize = rva(0x5B521C); // unk_5B521C
const VIDEO_COUNT_RVA: usize = rva(0x5ABC14); // dword_5ABC14
const SYSTEM_COUNT_RVA: usize = rva(0x5B5218); // dword_5B5218
const MOST: u32 = 0x140;
const LEVELS: u32 = 9;
const SIZES_RVA: usize = rva(0x5ABA98); // dword_5ABA98
const ALPHA_KIND_RVA: usize = rva(0x5ABA8C); // dword_5ABA8C
const FIRST_VIDEO_RVA: usize = rva(0x5ABA90); // dword_5ABA90
const FIRST_SYSTEM_RVA: usize = rva(0x5ABA94); // dword_5ABA94
const MIPS_RVA: usize = rva(0x5AB9A4); // dword_5AB9A4
const REPORTS_RVA: usize = rva(0x4F52AC); // dword_4F52AC
const IN_SCENE_RVA: usize = rva(0x5AB960); // dword_5AB960
const FOG_STATE_RVA: usize = rva(0x5AB9A0); // dword_5AB9A0: render state 0x1A's value
const CURRENT_TEXTURE_RVA: usize = rva(0x5AB93C); // g_currentTexture
const CLEARED_RVA: usize = rva(0x624138); // dword_624138: the rectangle to clear
const SCREEN_WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const SCREEN_HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const RELEASE_TEXT_RVA: usize = rva(0x5C2DDF); // "SVM: Release all textures"
const SET_STATE_RVA: usize = rva(0x41AB5B); // d3d::set_render_state
const SET_TEXTURE_RVA: usize = rva(0x41ACF5); // d3d::set_texture
const NO_TEXTURE: u32 = 0xFFFF_FC43;
/// A `DDERR`, for a surface that is not there.
const NOT_THERE: i32 = 0x8876_0000u32 as i32 | 0x1C2;

// ---- the texture formats ----

/// `d3d::find_texture_formats`, for a screen of `bits`.
pub unsafe fn describe_texture_formats(bits: u32) {
    // (bits, masks blue, green, red, alpha)
    let formats: [(u32, [u32; 4]); 4] = [
        (16, [0x001F, 0x03E0, 0x7C00, 0]),
        (16, [0x001F, 0x03E0, 0x7C00, 0x8000]),
        (16, [0x000F, 0x00F0, 0x0F00, 0xF000]),
        (32, [0xFF, 0xFF00, 0xFF_0000, 0xFF00_0000]),
    ];
    unsafe {
        for (rva, (format_bits, masks)) in DESCRIPTIONS_RVA.iter().zip(formats) {
            let description = game_ptr::<u32>(*rva);
            for n in 0..0x10 {
                description.add(n).write_unaligned(0);
            }
            description.write_unaligned(1);
            description.add(1).write_unaligned(format_bits / 8);
            for (n, mask) in masks.iter().enumerate() {
                description.add(3 + n).write_unaligned(*mask);
            }
            // DirectDraw's pixel format: size, flags (RGB, with alpha), bits, the
            // masks red, green, blue, alpha.
            description.add(8).write_unaligned(0x20);
            description
                .add(9)
                .write_unaligned(if masks[3] == 0 { 0x40 } else { 0x41 });
            description.add(11).write_unaligned(format_bits);
            description.add(12).write_unaligned(masks[2]);
            description.add(13).write_unaligned(masks[1]);
            description.add(14).write_unaligned(masks[0]);
            description.add(15).write_unaligned(masks[3]);
            let _ = describe_format(description.add(2));
        }
        let at = |n: usize| game_addr(DESCRIPTIONS_RVA[n]) as u32;
        let kinds = if bits == 32 {
            [at(3); 3]
        } else {
            [at(0), at(1), at(2)]
        };
        for (n, kind) in kinds.into_iter().enumerate() {
            game_write(KINDS_RVA + 4 + n * 4, kind);
        }
    }
}

// ---- the textures' memory ----

/// A record of a pool (0 system, 1 video).
fn record_at(pool: usize, number: u32) -> *mut TextureRecord {
    let base = if pool == 0 {
        SYSTEM_RECORDS_RVA
    } else {
        VIDEO_RECORDS_RVA
    };
    unsafe { (game_addr(base) as *mut TextureRecord).add(number as usize) }
}

struct Level {
    pixels: Vec<u8>,
    pitch: u32,
}

/// A surface's number: (pool, texture, level), the pool 0 for system, 1 for video.
fn surface_number(pool: usize, texture: u32, level: u32) -> usize {
    (pool * MOST as usize + texture as usize) * LEVELS as usize + level as usize
}

const SURFACES: usize = 2 * MOST as usize * LEVELS as usize;

static LEVELS_MEMORY: Mutex<Vec<Option<Level>>> = Mutex::new(Vec::new());
/// The surfaces' objects (a pointer to the table and the number each) and the table.
static OBJECTS: OnceLock<usize> = OnceLock::new();

fn objects() -> *mut [u32; 2] {
    *OBJECTS.get_or_init(|| {
        let mut table = vec![0u32; 0x40];
        table[0x64 / 4] = lock as *const () as u32;
        table[0x80 / 4] = unlock as *const () as u32;
        let table = Box::leak(table.into_boxed_slice()).as_ptr() as u32;
        let objects: Vec<[u32; 2]> = (0..SURFACES as u32).map(|n| [table, n]).collect();
        Box::leak(objects.into_boxed_slice()).as_mut_ptr() as usize
    }) as *mut [u32; 2]
}

fn object(number: usize) -> u32 {
    unsafe { objects().add(number) as u32 }
}

unsafe extern "system" fn lock(this: u32, _: u32, description: u32, _: u32, _: u32) -> i32 {
    let number = unsafe { ((this + 4) as *const u32).read_unaligned() } as usize;
    let mut memory = LEVELS_MEMORY.lock().unwrap();
    match memory.get_mut(number).and_then(|l| l.as_mut()) {
        Some(level) => unsafe {
            ((description + 0x24) as *mut u32).write_unaligned(level.pixels.as_mut_ptr() as u32);
            ((description + 0x10) as *mut u32).write_unaligned(level.pitch);
            0
        },
        None => NOT_THERE,
    }
}

unsafe extern "system" fn unlock(_: u32, _: u32) -> i32 {
    0
}

// ---- drawing ----

const KEPT_STATES_RVA: usize = rva(0x5AB6BC); // dword_5AB6BC: each state's last value
const STATES: usize = 0x40;
const TARGET_PIXELS_RVA: usize = rva(0x4F4988); // the screen's pixels (platform.rs)
const TARGET_PITCH_RVA: usize = rva(0x4F4984);
const SCREEN_BITS_RVA: usize = rva(0x4F4954); // g_screenBits

/// A fan into the screen, with the states and the current texture
/// (`g_currentTexture`, a video texture's number) as the game set them.
pub fn draw_fan(vertices: &[crate::rasteriser::Vertex]) {
    use crate::rasteriser::{States, Target, Texture};
    unsafe {
        let pixels: u32 = game_read(TARGET_PIXELS_RVA);
        if pixels == 0 {
            return;
        }
        let target = Target {
            pixels: pixels as *mut u8,
            pitch: game_read::<u32>(TARGET_PITCH_RVA) as usize,
            width: game_read::<u32>(SCREEN_WIDTH_RVA) as usize,
            height: game_read::<u32>(SCREEN_HEIGHT_RVA) as usize,
            bits: game_read(SCREEN_BITS_RVA),
        };
        let states: [u32; STATES] = game_read(KEPT_STATES_RVA);
        let current: u32 = game_read(CURRENT_TEXTURE_RVA);
        let memory = LEVELS_MEMORY.lock().unwrap();
        let texture = (current < MOST)
            .then(|| {
                let record = record_at(1, current).read_unaligned();
                let level = memory.get(surface_number(1, current, 0))?.as_ref()?;
                if record.format == 0 {
                    return None;
                }
                let format = (record.format as *const TextureFormat).read_unaligned();
                Some(Texture {
                    pixels: &level.pixels,
                    width: record.width as usize,
                    height: record.height as usize,
                    pitch: level.pitch as usize,
                    bytes: format.bytes_per_pixel as usize,
                    masks: format.format.masks,
                })
            })
            .flatten();
        with_depth(|depth| {
            crate::rasteriser::draw_fan(
                vertices,
                &target,
                &States(&states),
                texture.as_ref(),
                &mut depth.values,
            )
        });
    }
}

// ---- gfx::allocate_textures (0x4051ED) ----

static ALLOCATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static ALLOCATE: RawHook = RawHook::new(
    "gfx::allocate_textures",
    rva(0x4051ED),
    allocate_thunk,
    &ALLOCATE_ORIGINAL,
);
hook_asm!(
    allocate_thunk,
    allocate_original,
    dispatch = allocate,
    original = ALLOCATE_ORIGINAL
);

fn fatal(code: u32) -> FatalError {
    FatalError { code, message: "" }
}

/// A size by its number in `dword_5ABA98`.
unsafe fn size_of(code: u32) -> Option<u32> {
    unsafe {
        let entry = find_entry(code, game_addr(SIZES_RVA) as u32, 5).ok()??;
        Some(((entry + 4) as *const u32).read_unaligned())
    }
}

/// One texture more in a pool: its record made and its levels' memory.
unsafe fn make_texture(
    pool: usize,
    description: u32,
    width: u32,
    height: u32,
    levels: u32,
) -> Result<u32, FatalError> {
    let (count_rva, records_rva, full) = if pool == 0 {
        (SYSTEM_COUNT_RVA, SYSTEM_RECORDS_RVA, 0x80D1_8501)
    } else {
        (VIDEO_COUNT_RVA, VIDEO_RECORDS_RVA, 0x80D1_8601)
    };
    unsafe {
        let number: u32 = game_read(count_rva);
        if number >= MOST {
            return Err(fatal(full));
        }
        game_write(count_rva, number + 1);
        let at = game_addr(records_rva) as *mut TextureRecord;
        let at = at.add(number as usize);
        let mut record = at.read_unaligned();
        record.kind = if pool == 0 { 1 } else { 2 };
        record.width = width;
        record.height = height;
        record.format = description;
        record.levels = levels;
        let bytes = (description as *const TextureFormat)
            .read_unaligned()
            .bytes_per_pixel;
        let mut memory = LEVELS_MEMORY.lock().unwrap();
        if memory.is_empty() {
            memory.resize_with(SURFACES, || None);
        }
        for level in 0..LEVELS {
            let surface = surface_number(pool, number, level);
            if level < levels {
                let (w, h) = ((width >> level).max(1), (height >> level).max(1));
                memory[surface] = Some(Level {
                    pixels: vec![0; (w * h * bytes) as usize],
                    pitch: w * bytes,
                });
                record.surfaces[level as usize] = object(surface);
            } else {
                memory[surface] = None;
                record.surfaces[level as usize] = 0;
            }
        }
        if pool == 1 {
            record.holds = u32::MAX;
            record.texture = 1;
            record.handle = number + 1;
        }
        at.write_unaligned(record);
        Ok(number)
    }
}

unsafe fn allocate_textures(r: &Regs) -> Result<(), FatalError> {
    unsafe {
        let width = size_of(r.ecx & 0xFF).ok_or(fatal(0x80A1_8641))?;
        let height = if r.ecx & 0x1000 != 0 {
            size_of((r.ecx >> 8) & 0xFF).ok_or(fatal(0x80A1_8642))?
        } else {
            width
        };
        let description: u32 = game_read(KINDS_RVA + (r.edx as usize & 0xFF) * 4);
        game_write(ALPHA_KIND_RVA, (r.edx == 3) as u32);
        let levels = if game_read::<u32>(MIPS_RVA) == 1 {
            r.ebp
        } else {
            0
        }
        .wrapping_add(1);
        if levels > LEVELS {
            return Err(fatal(0x80A1_8643));
        }
        game_write(FIRST_VIDEO_RVA, game_read::<u32>(VIDEO_COUNT_RVA));
        game_write(FIRST_SYSTEM_RVA, game_read::<u32>(SYSTEM_COUNT_RVA));
        for _ in 0..r.eax {
            make_texture(1, description, width, height, levels)?;
        }
        let alpha: u32 = game_read(ALPHA_KIND_RVA);
        for _ in 0..r.ebx {
            let number = make_texture(0, description, width, height, levels)?;
            let at = record_at(0, number);
            let mut record = at.read_unaligned();
            record.keeps_alpha = alpha;
            at.write_unaligned(record);
        }
    }
    Ok(())
}

extern "C" fn allocate(r: &mut Regs) {
    ALLOCATE.count_call();
    let reports: u32 = unsafe { game_read(REPORTS_RVA) };
    unsafe { game_write(REPORTS_RVA, 0u32) };
    let done = unsafe { allocate_textures(r) };
    unsafe { game_write(REPORTS_RVA, reports) };
    match done {
        Ok(()) => unsafe {
            r.eax = game_read(FIRST_VIDEO_RVA);
            r.ebx = game_read(FIRST_SYSTEM_RVA);
            r.set_carry(false);
        },
        Err(e) => unsafe {
            crate::fatal::raise::refused(
                &ALLOCATE,
                crate::errors::Refused::Fatal(e),
                r,
                allocate_original,
            )
        },
    }
}

// ---- gfx::release_all_textures (0x4051D3) ----

static RELEASE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static RELEASE: RawHook = RawHook::new(
    "gfx::release_all_textures",
    rva(0x4051D3),
    release_thunk,
    &RELEASE_ORIGINAL,
);
hook_asm!(
    release_thunk,
    release_original,
    dispatch = release,
    original = RELEASE_ORIGINAL
);

/// `sub_41A3B2` for every texture of a pool, and the count 0.
unsafe fn release_pool(pool: usize) {
    let (count_rva, records_rva) = if pool == 0 {
        (SYSTEM_COUNT_RVA, SYSTEM_RECORDS_RVA)
    } else {
        (VIDEO_COUNT_RVA, VIDEO_RECORDS_RVA)
    };
    unsafe {
        let count: u32 = game_read(count_rva);
        let mut memory = LEVELS_MEMORY.lock().unwrap();
        for number in 0..count.min(MOST) {
            let at = (game_addr(records_rva) as *mut TextureRecord).add(number as usize);
            let mut record = at.read_unaligned();
            for level in 0..LEVELS {
                if let Some(slot) = memory.get_mut(surface_number(pool, number, level)) {
                    *slot = None;
                }
            }
            record.surfaces = [0; 9];
            record.textures = [0; 9];
            record.texture = 0;
            record.material = 0;
            record.kind = 0;
            record.levels = 0;
            record.holds = u32::MAX;
            at.write_unaligned(record);
        }
        game_write(count_rva, 0u32);
    }
}

extern "C" fn release(_r: &mut Regs) {
    RELEASE.count_call();
    unsafe {
        log_write(game_ptr(RELEASE_TEXT_RVA), true, 2);
        release_pool(1);
        release_pool(0);
    }
}

// ---- scene::load_texture (0x44E778) ----

static LOAD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static LOAD: RawHook = RawHook::new(
    "scene::load_texture",
    rva(0x44E778),
    load_thunk,
    &LOAD_ORIGINAL,
);
hook_asm!(
    load_thunk,
    load_original,
    dispatch = load,
    original = LOAD_ORIGINAL
);

/// eax = the system texture, ebx = the video one: the video one holds it, level for
/// level where both have the level.
extern "C" fn load(r: &mut Regs) {
    LOAD.count_call();
    let (system, video) = (r.eax, r.ebx);
    if system >= MOST || video >= MOST {
        log(&format!(
            "platform: scene::load_texture: texture {system:#x} into {video:#x} is past the records"
        ));
        return;
    }
    unsafe {
        let video_record = record_at(1, video);
        let mut held = video_record.read_unaligned();
        held.holds = system;
        video_record.write_unaligned(held);
        let levels = record_at(0, system).read_unaligned().levels.min(LEVELS);
        let mut memory = LEVELS_MEMORY.lock().unwrap();
        for level in 0..levels {
            let from = surface_number(0, system, level);
            let to = surface_number(1, video, level);
            let pixels = match memory.get(from).and_then(|l| l.as_ref()) {
                Some(level) => level.pixels.clone(),
                None => continue,
            };
            if let Some(Some(target)) = memory.get_mut(to) {
                let n = pixels.len().min(target.pixels.len());
                target.pixels[..n].copy_from_slice(&pixels[..n]);
            }
        }
    }
}

// ---- d3d::begin_scene (0x41ABC7), d3d::end_scene (0x41AB8A) ----

static BEGIN_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static BEGIN: RawHook = RawHook::new(
    "d3d::begin_scene",
    rva(0x41ABC7),
    begin_thunk,
    &BEGIN_ORIGINAL,
);
hook_asm!(
    begin_thunk,
    begin_original,
    dispatch = begin,
    original = BEGIN_ORIGINAL
);

/// The layer's own states at a scene's beginning: (value, state).
const BEGIN_STATES: [(u32, u32); 12] = [
    (1, 0x1F),
    (1, 0x07),
    (1, 0x0E),
    (2, 0x09),
    (4, 0x17),
    (1, 0x0F),
    (0, 0x18),
    (5, 0x19),
    (1, 0x04),
    (2, 0x11),
    (2, 0x12),
    (0, 0x23),
];

extern "C" fn begin(r: &mut Regs) {
    BEGIN.count_call();
    unsafe {
        game_write(IN_SCENE_RVA, 1u32);
        let mut work = *r;
        for (value, state) in BEGIN_STATES
            .into_iter()
            .chain([(game_read(FOG_STATE_RVA), 0x1A)])
        {
            work.eax = value;
            work.ebx = state;
            call_game(&mut work, game_addr(SET_STATE_RVA));
        }
        // `sub_41ACDE`: no texture is the current one, then none is set.
        game_write(CURRENT_TEXTURE_RVA, 0xFFFF_FC44u32);
        work.eax = NO_TEXTURE;
        call_game(&mut work, game_addr(SET_TEXTURE_RVA));
        r.eax = 0;
    }
    r.set_carry(false);
    r.set_zero(true);
}

static END_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static END: RawHook = RawHook::new("d3d::end_scene", rva(0x41AB8A), end_thunk, &END_ORIGINAL);
hook_asm!(
    end_thunk,
    end_original,
    dispatch = end,
    original = END_ORIGINAL
);

extern "C" fn end(r: &mut Regs) {
    END.count_call();
    unsafe {
        if game_read::<u32>(IN_SCENE_RVA) == 1 {
            game_write(IN_SCENE_RVA, 0u32);
            r.eax = 0;
            r.set_zero(true);
        }
    }
    r.set_carry(false);
}

// ---- d3d::clear_viewport (0x44E9D9) ----

static CLEAR_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static CLEAR: RawHook = RawHook::new(
    "d3d::clear_viewport",
    rva(0x44E9D9),
    clear_thunk,
    &CLEAR_ORIGINAL,
);
hook_asm!(
    clear_thunk,
    clear_original,
    dispatch = clear,
    original = CLEAR_ORIGINAL
);

/// The depth buffer, a float a pixel of the screen: 1.0 is the farthest.
pub struct Depth {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

pub static DEPTH: Mutex<Option<Depth>> = Mutex::new(None);

/// The depth buffer at the screen's size, made or made again.
pub fn with_depth<R>(f: impl FnOnce(&mut Depth) -> R) -> R {
    let (width, height) = unsafe {
        (
            game_read::<u32>(SCREEN_WIDTH_RVA) as usize,
            game_read::<u32>(SCREEN_HEIGHT_RVA) as usize,
        )
    };
    let mut depth = DEPTH.lock().unwrap();
    if depth
        .as_ref()
        .is_none_or(|d| d.width != width || d.height != height)
    {
        *depth = Some(Depth {
            width,
            height,
            values: vec![1.0; width * height],
        });
    }
    f(depth.as_mut().unwrap())
}

extern "C" fn clear(r: &mut Regs) {
    CLEAR.count_call();
    let [left, top, right, bottom]: [i32; 4] = unsafe { game_read(CLEARED_RVA) };
    with_depth(|depth| {
        let (right, bottom) = (
            (right.max(0) as usize).min(depth.width),
            (bottom.max(0) as usize).min(depth.height),
        );
        for y in (top.max(0) as usize)..bottom {
            for x in (left.max(0) as usize)..right {
                depth.values[y * depth.width + x] = 1.0;
            }
        }
    });
    r.eax = 0;
}

// None of these has an original to go back to.
const _: [unsafe extern "C" fn(*mut Regs); 5] = [
    release_original,
    load_original,
    begin_original,
    end_original,
    clear_original,
];

pub static HOOKS: &[&dyn GameHook] = &[&ALLOCATE, &RELEASE, &LOAD, &BEGIN, &END, &CLEAR];
