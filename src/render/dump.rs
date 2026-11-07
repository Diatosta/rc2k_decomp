//! A frame's commands written to a file, for replaying them outside the program.

use super::{Command, Target};

/// With `RC2K_DUMP=n` in the environment: the n-th flush of 1000 fans or more written to
/// `rc2k_frame_dump.bin`, for replaying it outside the program (the benchmark): "RC2KDUMP",
/// the screen's width, height, bits; the textures (each once: width, height, pitch, bytes,
/// four masks, its bytes' length and bytes); then each noted thing in order: 1 a fan (its
/// vertices' count and the vertices, 8 dwords each; the 0x40 states; its texture's number
/// or -1), 2 a depth clear (left, top, right, bottom). All little-endian dwords.
pub fn dump_if_asked(noted: &[Command], into: &Target) {
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static SEEN: AtomicU32 = AtomicU32::new(0);
    let Some(wanted) = std::env::var("RC2K_DUMP")
        .ok()
        .and_then(|n| n.parse::<u32>().ok())
    else {
        return;
    };
    let fans = noted
        .iter()
        .filter(|n| matches!(n, Command::Fan { .. }))
        .count();
    if fans < 1000 || SEEN.fetch_add(1, Relaxed) + 1 != wanted {
        return;
    }
    let mut out: Vec<u8> = b"RC2KDUMP".to_vec();
    let put = |out: &mut Vec<u8>, v: u32| out.extend(v.to_le_bytes());
    for v in [into.width as u32, into.height as u32, into.bits] {
        put(&mut out, v);
    }
    let mut textures: Vec<crate::pointers::GamePtr> = Vec::new();
    for n in noted {
        if let Command::Fan {
            texture: Some(t), ..
        } = n
            && !textures.contains(&t.pixels)
        {
            textures.push(t.pixels);
        }
    }
    put(&mut out, textures.len() as u32);
    for &at in &textures {
        let t = noted
            .iter()
            .find_map(|n| match n {
                Command::Fan {
                    texture: Some(t), ..
                } if t.pixels == at => Some(t),
                _ => None,
            })
            .unwrap();
        for v in [t.width, t.height, t.pitch, t.bytes] {
            put(&mut out, v as u32);
        }
        for m in t.masks {
            put(&mut out, m);
        }
        put(&mut out, t.len as u32);
        let what = "a noted texture's pixels";
        out.extend(
            t.pixels
                .read_bytes(t.len, what)
                .unwrap_or_else(|e| crate::fatal::raise::end(what, e)),
        );
    }
    put(&mut out, noted.len() as u32);
    for n in noted {
        match n {
            Command::Fan {
                vertices,
                states,
                texture,
                ..
            } => {
                put(&mut out, 1);
                put(&mut out, vertices.len() as u32);
                for v in vertices {
                    for &d in v {
                        put(&mut out, d);
                    }
                }
                for &st in states.iter() {
                    put(&mut out, st);
                }
                let number = texture
                    .as_ref()
                    .and_then(|t| textures.iter().position(|&p| p == t.pixels))
                    .map_or(u32::MAX, |n| n as u32);
                put(&mut out, number);
            }
            Command::Clear(r) => {
                put(&mut out, 2);
                for &v in r {
                    put(&mut out, v as u32);
                }
            }
        }
    }
    let written = std::fs::write("rc2k_frame_dump.bin", &out);
    crate::framework::log(&format!(
        "platform_3d: flush {wanted} of {fans} fans dumped to rc2k_frame_dump.bin ({} bytes, {} textures): {written:?}",
        out.len(),
        textures.len()
    ));
}

/// With `RC2K_FAN_HASH` in the environment: each race flush's commands hashed (every fan's
/// vertices, states and texture, every clear) and logged with the flush's number, so that two
/// runs of a replay can be compared flush by flush (the check of a change that should draw the
/// same).
pub fn hash_if_asked(noted: &[Command]) {
    use std::hash::{Hash, Hasher};
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    static FLUSHES: AtomicU32 = AtomicU32::new(0);
    if !*ON.get_or_init(|| std::env::var_os("RC2K_FAN_HASH").is_some())
        || crate::race::owner::race().course.numbers.in_race.get() == 0
    {
        return;
    }
    let mut h = std::collections::hash_map::DefaultHasher::new();
    // A texture by its pixels, not by where they are: that differs from run to run.
    let mut seen: Vec<(u32, u64)> = Vec::new();
    let mut texture_hash = |t: &super::TextureNote| {
        let at = t.pixels.addr();
        if let Some(&(_, v)) = seen.iter().find(|s| s.0 == at) {
            return v;
        }
        let mut th = std::collections::hash_map::DefaultHasher::new();
        t.pixels
            .read_bytes(t.len, "a hashed texture")
            .ok()
            .hash(&mut th);
        let v = th.finish();
        seen.push((at, v));
        v
    };
    for n in noted {
        match n {
            Command::Fan {
                vertices,
                states,
                texture,
                ..
            } => {
                1u8.hash(&mut h);
                for v in vertices {
                    v.hash(&mut h);
                }
                states.hash(&mut h);
                texture.as_ref().map(&mut texture_hash).hash(&mut h);
            }
            Command::Clear(rect) => {
                2u8.hash(&mut h);
                rect.hash(&mut h);
            }
        }
    }
    let n = FLUSHES.fetch_add(1, Relaxed);
    crate::framework::log(&format!(
        "fan hash {n}: {} commands, {:016x}",
        noted.len(),
        h.finish()
    ));
}
