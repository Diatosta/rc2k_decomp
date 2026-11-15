//! A diagnostic (`RC2K_SEETHRU_AREA=1`): which see-through (blended) fans cover the most pixels.
//!
//! Every list of noted commands is looked at on the game's thread as it is handed to the GPU
//! (both backends: `gpu::draw`, the handheld's `gles` draw). For each fan the pixels its
//! triangles cover on the target are summed, clipped to the target, and added to the fan's
//! texture and to its kind (the blend factors, the depth write, whether it is textured, fogged
//! or alpha tested). Overlap between fans is counted again: the figure is the fragments
//! the GPU is asked for, which is what costs. Every 5 seconds the top 12 see-through textures
//! are logged with the totals for solid and for all see-through fans.
//!
//! `RC2K_SEETHRU_DUMP=<folder>` also saves the 8 biggest see-through textures of the first
//! report as PNG files (colour, and the alpha channel as grey) the first time a frame reads
//! them afterwards. With the variable unset nothing is read or counted: the switch is one
//! atomic load per list and per frame.

use super::{Command, CommandList, TextureId, TextureNote};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
use std::time::{Duration, Instant};

/// How often the figures are logged.
const REPORT_EVERY: Duration = Duration::from_secs(5);
/// Rows in the log's tables, and textures saved.
const TOP_ROWS: usize = 12;
const DUMPED: usize = 8;

/// 0: not looked at yet; 1: off; 2: on.
static SWITCH: AtomicU8 = AtomicU8::new(0);

fn enabled() -> bool {
    match SWITCH.load(Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let on = std::env::var("RC2K_SEETHRU_AREA").is_ok_and(|v| v != "0" && !v.is_empty());
            SWITCH.store(if on { 2 } else { 1 }, Relaxed);
            on
        }
    }
}

/// The pixels a triangle covers on a target of `size`, its corners as the game gives them
/// (the places the GPU's shaders use for x and y), clipped to the target. Zero for a corner
/// that is not a finite number.
pub fn triangle_pixels(corners: [(f64, f64); 3], size: (f64, f64)) -> f64 {
    if corners
        .iter()
        .any(|(x, y)| !x.is_finite() || !y.is_finite())
    {
        return 0.0;
    }
    // Sutherland-Hodgman against the four edges of the target, then the polygon's area.
    let mut polygon: Vec<(f64, f64)> = corners.to_vec();
    type Inside = fn((f64, f64), (f64, f64)) -> bool;
    type Cut = fn((f64, f64), (f64, f64), (f64, f64)) -> (f64, f64);
    let edges: [(Inside, Cut); 4] = [
        (|p, _| p.0 >= 0.0, |a, b, _| cut_x(a, b, 0.0)),
        (|p, s| p.0 <= s.0, |a, b, s| cut_x(a, b, s.0)),
        (|p, _| p.1 >= 0.0, |a, b, _| cut_y(a, b, 0.0)),
        (|p, s| p.1 <= s.1, |a, b, s| cut_y(a, b, s.1)),
    ];
    for (inside, cut) in edges {
        let from = std::mem::take(&mut polygon);
        for (i, &b) in from.iter().enumerate() {
            let a = from[(i + from.len() - 1) % from.len()];
            match (inside(a, size), inside(b, size)) {
                (true, true) => polygon.push(b),
                (true, false) => polygon.push(cut(a, b, size)),
                (false, true) => {
                    polygon.push(cut(a, b, size));
                    polygon.push(b);
                }
                (false, false) => {}
            }
        }
        if polygon.len() < 3 {
            return 0.0;
        }
    }
    let twice: f64 = (0..polygon.len())
        .map(|i| {
            let (a, b) = (polygon[i], polygon[(i + 1) % polygon.len()]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum();
    twice.abs() / 2.0
}

/// Where the segment `a`-`b` crosses the vertical line `x`.
fn cut_x(a: (f64, f64), b: (f64, f64), x: f64) -> (f64, f64) {
    (x, a.1 + (b.1 - a.1) * (x - a.0) / (b.0 - a.0))
}

/// Where the segment `a`-`b` crosses the horizontal line `y`.
fn cut_y(a: (f64, f64), b: (f64, f64), y: f64) -> (f64, f64) {
    (a.0 + (b.0 - a.0) * (y - a.1) / (b.1 - a.1), y)
}

#[derive(Clone, Copy, Default)]
struct Sum {
    fans: u64,
    triangles: u64,
    pixels: f64,
}

impl Sum {
    fn add(&mut self, fans: u64, triangles: u64, pixels: f64) {
        self.fans += fans;
        self.triangles += triangles;
        self.pixels += pixels;
    }
}

#[derive(Default)]
struct State {
    window_began: Option<Instant>,
    frames: u64,
    size: (u32, u32),
    solid: Sum,
    see_through: Sum,
    /// Of the see-through: fans whose texture has a full alpha in every pixel (the blend
    /// changes nothing there), fans whose texture has some, and fans with no texture.
    opaque_texture: Sum,
    alpha_texture: Sum,
    plain: Sum,
    /// Of the see-through: fans whose blend changes no pixel (the blend keeps the source's
    /// colour, and the texture and every corner's colour have a full alpha), and the rest,
    /// which really show what is behind them.
    no_effect: Sum,
    shows_behind: Sum,
    /// Whether each texture seen has a full alpha everywhere (looked at once, when first read).
    opaque: HashMap<TextureId, bool>,
    /// By texture (`None`: no texture), with the last note of it for its size and format.
    textures: HashMap<Option<TextureId>, (Option<TextureNote>, Sum, Sum)>,
    kinds: HashMap<String, Sum>,
    reports: u32,
    dumped: bool,
    /// The textures to save, those saved already, and where.
    to_dump: Vec<TextureId>,
    dump_folder: Option<std::path::PathBuf>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

/// What a fan's states say apart from its texture: the blend's factors, whether it writes
/// depth, and whether it is textured, fogged and alpha tested.
fn kind_of(states: &[u32; super::STATES], textured: bool) -> String {
    format!(
        "blend {}/{} depth_write={} {} fog={} alpha_test={}",
        states[0x13],
        states[0x14],
        u8::from(states[0x0E] != 0),
        if textured { "textured" } else { "plain" },
        u8::from(states[0x1C] != 0),
        u8::from(states[0x0F] != 0),
    )
}

/// A list of commands on its way to the GPU, `width` x `height` the target.
pub fn note_list(list: &CommandList, width: u32, height: u32) {
    if !enabled() {
        return;
    }
    let mut guard = STATE.lock().unwrap();
    let state = guard.get_or_insert_with(|| State {
        dump_folder: std::env::var_os("RC2K_SEETHRU_DUMP").map(Into::into),
        ..State::default()
    });
    state.size = (width, height);
    state.window_began.get_or_insert_with(Instant::now);
    let size = (f64::from(width), f64::from(height));
    for command in &list.commands {
        let Command::Fan {
            vertices,
            states,
            texture,
            ..
        } = command
        else {
            continue;
        };
        let fan = list.fan_vertices(*vertices);
        if fan.len() < 3 {
            continue;
        }
        let place = |v: &[u32; 8]| {
            (
                f64::from(f32::from_bits(v[0])),
                f64::from(f32::from_bits(v[1])),
            )
        };
        let first = place(&fan[0]);
        let mut pixels = 0.0;
        for n in 1..fan.len() - 1 {
            pixels += triangle_pixels([first, place(&fan[n]), place(&fan[n + 1])], size);
        }
        let triangles = fan.len() as u64 - 2;
        if states[0x1B] == 0 {
            state.solid.add(1, triangles, pixels);
            continue;
        }
        state.see_through.add(1, triangles, pixels);
        let keeps_source = matches!((states[0x13], states[0x14]), (5, 6) | (2, 1));
        let full_alpha = fan.iter().all(|v| v[4] >> 24 == 0xFF);
        let texture_full = texture.is_none_or(|note| {
            *state
                .opaque
                .entry(note.pixels)
                .or_insert_with(|| all_opaque(&super::texture_as_bgra(&note)))
        });
        if keeps_source && full_alpha && texture_full {
            state.no_effect.add(1, triangles, pixels);
        } else {
            state.shows_behind.add(1, triangles, pixels);
        }
        match texture {
            None => state.plain.add(1, triangles, pixels),
            Some(note) => {
                let opaque = *state
                    .opaque
                    .entry(note.pixels)
                    .or_insert_with(|| all_opaque(&super::texture_as_bgra(note)));
                if opaque {
                    state.opaque_texture.add(1, triangles, pixels);
                } else {
                    state.alpha_texture.add(1, triangles, pixels);
                }
            }
        }
        state
            .kinds
            .entry(kind_of(states, texture.is_some()))
            .or_default()
            .add(1, triangles, pixels);
        let id = texture.map(|t| t.pixels);
        let entry = state.textures.entry(id).or_default();
        entry.0 = *texture;
        entry.1.add(1, triangles, pixels);
        if !(keeps_source && full_alpha && texture_full) {
            entry.2.add(1, triangles, pixels);
        }
        if let (Some(note), Some(id)) = (texture, id)
            && let Some(at) = state.to_dump.iter().position(|d| *d == id)
        {
            state.to_dump.swap_remove(at);
            if let Some(folder) = &state.dump_folder {
                save_texture(folder, state.reports, note);
            }
        }
    }
}

/// A frame is over: counted, and the figures logged when the window is up.
pub fn frame_end() {
    if !enabled() {
        return;
    }
    let mut guard = STATE.lock().unwrap();
    let Some(state) = guard.as_mut() else {
        return;
    };
    state.frames += 1;
    let Some(began) = state.window_began else {
        return;
    };
    if began.elapsed() < REPORT_EVERY {
        return;
    }
    report(state, began.elapsed());
    state.window_began = None;
    state.frames = 0;
    state.solid = Sum::default();
    state.see_through = Sum::default();
    state.opaque_texture = Sum::default();
    state.alpha_texture = Sum::default();
    state.plain = Sum::default();
    state.no_effect = Sum::default();
    state.shows_behind = Sum::default();
    state.textures.clear();
    state.kinds.clear();
}

fn report(state: &mut State, elapsed: Duration) {
    let frames = state.frames.max(1) as f64;
    let screen = f64::from(state.size.0.max(1)) * f64::from(state.size.1.max(1));
    let log = |line: String| crate::logfile::program_log::log(&line);
    let row = |what: &str, s: &Sum| {
        format!(
            "seethru: {what}: {:.1} fans, {:.1} triangles, {:.0} pixels = {:.2} screens a frame",
            s.fans as f64 / frames,
            s.triangles as f64 / frames,
            s.pixels / frames,
            s.pixels / frames / screen,
        )
    };
    log(format!(
        "seethru: window {:.1} s, {} frames, target {}x{}",
        elapsed.as_secs_f64(),
        state.frames,
        state.size.0,
        state.size.1
    ));
    log(row("solid", &state.solid));
    log(row("see-through", &state.see_through));
    log(row("  of it, blend changes nothing", &state.no_effect));
    log(row("  of it, shows what is behind", &state.shows_behind));
    log(row("  of it, texture opaque", &state.opaque_texture));
    log(row("  of it, texture with alpha", &state.alpha_texture));
    log(row("  of it, no texture", &state.plain));
    let mut by_texture: Vec<_> = state.textures.iter().collect();
    let table = |title: &str, state: &State, by_texture: &mut Vec<_>, behind: bool| {
        by_texture.sort_by(
            |a: &(&Option<TextureId>, &(Option<TextureNote>, Sum, Sum)), b| {
                let key = |t: &(&Option<TextureId>, &(Option<TextureNote>, Sum, Sum))| {
                    if behind { t.1.2.pixels } else { t.1.1.pixels }
                };
                key(b).total_cmp(&key(a))
            },
        );
        log(format!("seethru: top textures by {title}"));
        for (id, (note, sum, hidden)) in by_texture.iter().take(TOP_ROWS) {
            let what = match (id, note) {
                (Some(id), Some(n)) => format!(
                    "texture {}.{} {}x{} {}B/px{}{}",
                    id.surface,
                    id.serial,
                    n.width,
                    n.height,
                    n.bytes,
                    if n.masks[3] != 0 { " alpha" } else { "" },
                    match state.opaque.get(id) {
                        Some(true) => " (all opaque)",
                        Some(false) => " (real alpha)",
                        None => "",
                    }
                ),
                _ => "no texture".to_string(),
            };
            log(format!(
                "{}; of it shows what is behind: {:.2} screens",
                row(&what, sum),
                hidden.pixels / frames / screen
            ));
        }
    };
    table("area", state, &mut by_texture, false);
    table(
        "area that shows what is behind",
        state,
        &mut by_texture,
        true,
    );
    let first_dumped: Vec<TextureId> = by_texture
        .iter()
        .filter_map(|(id, _)| **id)
        .take(DUMPED)
        .collect();
    let mut kinds: Vec<_> = state.kinds.iter().collect();
    kinds.sort_by(|a, b| b.1.pixels.total_cmp(&a.1.pixels));
    for (kind, sum) in kinds.iter().take(6) {
        log(row(kind, sum));
    }
    state.reports += 1;
    // The first window that has real see-through work (not a menu's): its top textures are saved.
    let busy = state.see_through.pixels / frames / screen >= 1.0;
    if busy && !state.dumped && state.dump_folder.is_some() {
        state.dumped = true;
        state.to_dump = first_dumped;
        if let Some(folder) = &state.dump_folder {
            let _ = std::fs::create_dir_all(folder);
        }
    }
}

/// Whether every pixel of `bgra` (4 bytes each) has a full alpha.
fn all_opaque(bgra: &[u8]) -> bool {
    bgra.as_chunks::<4>().0.iter().all(|p| p[3] == 255)
}

/// `note`'s pixels as `<folder>/<surface>_<serial>_<w>x<h>.png` and its alpha as `_a.png`.
fn save_texture(folder: &std::path::Path, _report: u32, note: &TextureNote) {
    let bgra = super::texture_as_bgra(note);
    let name = format!(
        "{}_{}_{}x{}",
        note.pixels.surface, note.pixels.serial, note.width, note.height
    );
    let pixels = bgra.as_chunks::<4>().0;
    let rgb: Vec<u8> = pixels.iter().flat_map(|p| [p[2], p[1], p[0]]).collect();
    let alpha: Vec<u8> = pixels.iter().map(|p| p[3]).collect();
    let write = |file: String, bytes: Vec<u8>| {
        let _ = std::fs::write(folder.join(file), bytes);
    };
    write(
        format!("{name}.png"),
        png(note.width as u32, note.height as u32, 2, &rgb),
    );
    write(
        format!("{name}_a.png"),
        png(note.width as u32, note.height as u32, 0, &alpha),
    );
    crate::logfile::program_log::log(&format!("seethru: saved texture {name}"));
}

/// A PNG of 8-bit samples: `colour_type` 2 is RGB, 0 grey; rows unfiltered.
fn png(width: u32, height: u32, colour_type: u8, samples: &[u8]) -> Vec<u8> {
    let per_pixel = if colour_type == 2 { 3 } else { 1 };
    let row = width as usize * per_pixel;
    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    for line in samples.chunks_exact(row.max(1)) {
        raw.push(0);
        raw.extend_from_slice(line);
    }
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend(width.to_be_bytes());
    header.extend(height.to_be_bytes());
    header.extend([8, colour_type, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(
        &mut out,
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6),
    );
    chunk(&mut out, b"IEND", &[]);
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend(kind);
    out.extend(data);
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in &out[start..] {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    out.extend((!crc).to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_triangle_inside_the_target_has_its_own_area() {
        let a = triangle_pixels([(10.0, 10.0), (30.0, 10.0), (10.0, 20.0)], (100.0, 100.0));
        assert!((a - 100.0).abs() < 1e-9, "{a}");
    }

    #[test]
    fn a_triangle_past_the_edge_is_clipped() {
        // A 20 x 10 right triangle with its right-angle corner at x = 90: the target ends at 100,
        // so what lies in x <= 100 is the trapezoid between x = 90 and 100.
        let whole = triangle_pixels([(90.0, 0.0), (110.0, 0.0), (90.0, 10.0)], (200.0, 100.0));
        assert!((whole - 100.0).abs() < 1e-9);
        let clipped = triangle_pixels([(90.0, 0.0), (110.0, 0.0), (90.0, 10.0)], (100.0, 100.0));
        // Height at x = 100 is 5: area = (10 + 5) / 2 * 10.
        assert!((clipped - 75.0).abs() < 1e-9, "{clipped}");
    }

    #[test]
    fn a_triangle_off_the_target_or_not_a_number_covers_nothing() {
        assert_eq!(
            triangle_pixels([(-50.0, 5.0), (-10.0, 5.0), (-30.0, 20.0)], (100.0, 100.0)),
            0.0
        );
        assert_eq!(
            triangle_pixels([(f64::NAN, 5.0), (10.0, 5.0), (30.0, 20.0)], (100.0, 100.0)),
            0.0
        );
    }

    #[test]
    fn a_triangle_covering_the_target_is_clipped_to_all_of_it() {
        let a = triangle_pixels(
            [(-500.0, -500.0), (1500.0, -500.0), (-500.0, 1500.0)],
            (100.0, 50.0),
        );
        assert!((a - 5000.0).abs() < 1e-6, "{a}");
    }

    #[test]
    fn the_png_has_the_signature_and_a_valid_crc_for_its_end() {
        let p = png(2, 1, 2, &[1, 2, 3, 4, 5, 6]);
        assert_eq!(&p[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&p[p.len() - 8..p.len() - 4], b"IEND");
        assert_eq!(&p[p.len() - 4..], &[0xAE, 0x42, 0x60, 0x82]);
    }
}
