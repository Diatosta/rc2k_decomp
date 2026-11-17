//! The handheld GPU's jobs, and the step between the game's thread and the GL's: each part's
//! commands made into batches (`Batches::of_layered`) on a thread of their own, one part ahead
//! of the GL's drawing, so the GL's thread only brings textures up, binds and draws (batching
//! took about a third of its time). The one fact the batches need from the GL's texture cache,
//! whether a texture is opaque, is kept here from the same jobs in the same order (`Opacity`):
//! the batches, and so the picture, are the same as the GL's thread would make.

use super::batches::Batches;
use super::{CommandList, TextureId, TextureMap, TextureNote};
use crate::gfx::draw2d::quads::{self, Taken};
use std::sync::mpsc::Sender;

/// The canvas's written rows, copied.
pub struct Written {
    pub first: usize,
    pub last: usize,
    pub bytes: Vec<u8>,
}

/// A part's retained background (`background`): drawn under the canvas's rows.
pub enum Background {
    /// The GL's background texture holds the picture already.
    Held,
    /// The picture's bytes (the screen's rows, 32 bits), to put in the texture first.
    Upload(Vec<u8>),
}

/// What a draw needs, all of it the threads' own: the game goes on with its memory meanwhile.
pub struct Part {
    /// Given back (`render::give_back`) where it was last read: the GL's thread, after drawing.
    pub commands: CommandList,
    /// The textures to bring up before the part is drawn, each with its whole picture
    /// (`texture_as_bgra`), in the order they are brought up.
    pub uploads: Vec<(TextureNote, Vec<u8>)>,
    pub written: Vec<Written>,
    /// The 2D quads noted since the last part (`gfx::draw2d::quads`) and the atlas texels to
    /// send before they are drawn.
    pub quads: Taken,
    /// The background under the canvas's rows, if the frame has one.
    pub background: Option<Background>,
    pub size: (i32, i32),
    /// The percent of the screen's size the part's 3D is drawn at (`render::flush_scale`).
    pub scale: u32,
}

/// A part made ready to draw.
pub struct Batched {
    pub part: Part,
    pub batches: Batches,
    /// The quads' vertices (`quads::vertices`), made here, off the GL's thread.
    pub quad_vertices: Vec<[u32; quads::VERTEX_WORDS]>,
    /// Whether each of `batches.textures` is opaque, in its order (the GL bench's comparison
    /// with one texture a draw reads them).
    pub opaque: Vec<bool>,
}

/// A job for the GL: from the game's thread with its part as commands (`Job<Part>`), to the
/// GL's with it batched (`Job<Batched>`).
pub enum Job<Draw> {
    /// The textures whose memory is gone, by address.
    Forget(Vec<TextureId>),
    Draw(Draw),
    ShowTarget,
    ShowPicture {
        frame: Vec<u8>,
        width: i32,
        height: i32,
    },
    ReadTarget(Sender<Vec<u8>>),
}

/// Whether the batches are told what their draws can leave out (`Batches::of_layered`'s
/// `fast_path`): unless `RC2K_FAST_PATH=0`, read once.
pub fn fast_path_wanted() -> bool {
    static WANTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *WANTED.get_or_init(|| std::env::var("RC2K_FAST_PATH").map_or(true, |v| v != "0"))
}

/// Whether every pixel's alpha of a texture's picture (`texture_as_bgra`'s bytes) is full: a
/// first pass over it can take a second (`Batches::of`).
pub fn opaque_picture(bytes: &[u8]) -> bool {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .all(|pixel| pixel[3] == 0xFF)
}

/// Whether each texture the GL's cache holds is opaque, as it holds it when it draws the part
/// being batched. The GL's cache takes a texture in when a part brings it up (before the part's
/// batches are drawn) and lets it go at a `Forget`; this sees the same jobs in the same order
/// and does the same, so for every part the two agree (a texture the cache does not hold is
/// not opaque, as before).
#[derive(Default)]
pub struct Opacity(TextureMap<bool>);

impl Opacity {
    fn forget(&mut self, gone: &[TextureId]) {
        for pixels in gone {
            self.0.remove(pixels);
        }
    }

    fn brought_up(&mut self, note: &TextureNote, bytes: &[u8]) {
        self.0.insert(note.pixels, opaque_picture(bytes));
    }

    fn of(&self, note: &TextureNote) -> bool {
        self.0.get(&note.pixels).copied().unwrap_or(false)
    }
}

/// The game's job as the GL's: a part's textures noted as brought up, then its batches made;
/// the textures gone noted as let go of; the rest as they are. The part's commands are of no
/// use to the GL once batched unless `keep_commands`: they are emptied here, so that the
/// render states they share are let go of on this thread and not on the GL's.
pub fn batched(
    opacity: &mut Opacity,
    job: Job<Part>,
    same_array: fn(&TextureNote, &TextureNote) -> bool,
    keep_commands: bool,
) -> Job<Batched> {
    match job {
        Job::Draw(mut part) => {
            for (note, bytes) in &part.uploads {
                opacity.brought_up(note, bytes);
            }
            let (width, height) = part.size;
            let opaque = |note: &TextureNote| opacity.of(note);
            let batches = Batches::of_layered(
                &part.commands,
                width as u32,
                height as u32,
                &opaque,
                same_array,
                fast_path_wanted(),
            );
            let opaque = batches
                .textures
                .iter()
                .map(|note| opacity.of(note))
                .collect();
            let quad_vertices = quads::vertices(&part.quads.rects);
            if !keep_commands {
                part.commands.clear();
            }
            Job::Draw(Batched {
                part,
                batches,
                quad_vertices,
                opaque,
            })
        }
        Job::Forget(gone) => {
            opacity.forget(&gone);
            Job::Forget(gone)
        }
        Job::ShowTarget => Job::ShowTarget,
        Job::ShowPicture {
            frame,
            width,
            height,
        } => Job::ShowPicture {
            frame,
            width,
            height,
        },
        Job::ReadTarget(answer) => Job::ReadTarget(answer),
    }
}

/// The batching thread: the game's jobs, in order, batched and handed to the GL's thread. It
/// ends when the game's side is dropped or the GL's thread has ended.
#[cfg(not(target_os = "emscripten"))]
pub fn run(
    jobs: std::sync::mpsc::Receiver<Job<Part>>,
    gl: std::sync::mpsc::SyncSender<Job<Batched>>,
    same_array: fn(&TextureNote, &TextureNote) -> bool,
    keep_commands: bool,
) {
    let mut opacity = Opacity::default();
    for job in jobs {
        let start = std::time::Instant::now();
        let job = batched(&mut opacity, job, same_array, keep_commands);
        super::GL_BATCHING_MICROS.fetch_add(
            start.elapsed().as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        if gl.send(job).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::batches::NO_TEXTURE;
    use super::super::{Command, STATES, TestCommand, list};
    use super::*;

    /// A small linear congruential generator: the same streams on every run.
    struct Numbers(u64);

    impl Numbers {
        fn below(&mut self, n: u32) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) % u64::from(n)) as u32
        }
    }

    fn note(surface: u32, width: usize) -> TextureNote {
        TextureNote {
            pixels: TextureId { surface, serial: 0 },
            len: width * width * 4,
            width,
            height: width,
            pitch: width * 4,
            bytes: 4,
            masks: [0xFF_0000, 0xFF00, 0xFF, 0xFF00_0000],
        }
    }

    /// A picture of `note`'s size, opaque or with one pixel not.
    fn picture(note: &TextureNote, opaque: bool) -> Vec<u8> {
        let mut bytes = vec![0xFF; note.width * note.height * 4];
        if !opaque {
            bytes[7] = 0x80;
        }
        bytes
    }

    fn same_size(a: &TextureNote, b: &TextureNote) -> bool {
        (a.width, a.height) == (b.width, b.height)
    }

    /// A fan over a few corners near (`x`, `y`), opaque or not, of one of a few kinds of
    /// states: plain, a second pass's (no depth writes, blended by alpha), alpha tested, fogged.
    fn fan(numbers: &mut Numbers, texture: Option<TextureNote>, at: (u32, u32)) -> TestCommand {
        let mut states = [0; STATES];
        states[0x07] = 1;
        states[0x17] = 4;
        match numbers.below(4) {
            0 => states[0x0E] = 1,
            1 => {
                states[0x1B] = 1;
                (states[0x13], states[0x14]) = (5, 6);
            }
            2 => {
                states[0x0E] = 1;
                states[0x0F] = 1;
                states[0x19] = 7;
                states[0x18] = 0x80;
            }
            _ => {
                states[0x0E] = 1;
                states[0x1C] = 1;
                states[0x22] = 0xFF80_4020;
            }
        }
        let alpha = if numbers.below(4) == 0 { 0x80 } else { 0xFF };
        let corners = 3 + numbers.below(3) as usize;
        let vertices = (0..corners)
            .map(|n| {
                let x = (at.0 + 8 * (n as u32 % 2)) as f32;
                let y = (at.1 + 8 * (n as u32 / 2)) as f32;
                [
                    x.to_bits(),
                    y.to_bits(),
                    0.5f32.to_bits(),
                    1f32.to_bits(),
                    alpha << 24 | 0x80_8080,
                    0,
                    0,
                    0,
                ]
            })
            .collect();
        TestCommand::Fan {
            vertices,
            states: std::sync::Arc::new(states),
            texture,
            bounds: (0.0, 0.0, 0.0, 0.0),
        }
    }

    /// A stream of the game's jobs: parts of fans over textures of three sizes, drawn twice
    /// running at times over the same places (a second pass), with depth clears between; their
    /// textures brought up as the game's side does (when its picture is new or written), some
    /// written over between parts with a picture whose opacity differs, some gone.
    fn stream(seed: u64) -> Vec<Job<Part>> {
        let mut numbers = Numbers(seed);
        let notes: Vec<TextureNote> = (0..12)
            .map(|n| note(n, [8, 16, 32][n as usize % 3]))
            .collect();
        let mut up = std::collections::HashSet::new();
        let mut jobs = Vec::new();
        for _ in 0..40 {
            let gone: Vec<TextureId> = (0..numbers.below(3))
                .map(|_| notes[numbers.below(12) as usize].pixels)
                .collect();
            let written: Vec<TextureId> = (0..numbers.below(3))
                .map(|_| notes[numbers.below(12) as usize].pixels)
                .collect();
            for pixels in gone.iter().chain(&written) {
                up.remove(pixels);
            }
            if !gone.is_empty() {
                jobs.push(Job::Forget(gone));
            }
            let mut commands = Vec::new();
            for _ in 0..numbers.below(200) {
                if numbers.below(30) == 0 {
                    let left = numbers.below(320) as usize;
                    let top = numbers.below(240) as usize;
                    commands.push(TestCommand::Clear([left, top, left + 40, top + 30]));
                    continue;
                }
                let texture = (numbers.below(6) != 0).then(|| notes[numbers.below(12) as usize]);
                let at = (numbers.below(300), numbers.below(220));
                let first = fan(&mut numbers, texture, at);
                let again = numbers.below(3) == 0;
                commands.push(first);
                if again {
                    // The same places again: a second pass when the states fit.
                    let Some(TestCommand::Fan { vertices, .. }) = commands.last() else {
                        unreachable!()
                    };
                    let vertices = vertices.clone();
                    let texture =
                        (numbers.below(4) != 0).then(|| notes[numbers.below(12) as usize]);
                    let TestCommand::Fan {
                        states, texture: t, ..
                    } = fan(&mut numbers, texture, at)
                    else {
                        unreachable!()
                    };
                    commands.push(TestCommand::Fan {
                        vertices,
                        states,
                        texture: t,
                        bounds: (0.0, 0.0, 0.0, 0.0),
                    });
                }
            }
            let commands = list(commands);
            let mut uploads = Vec::new();
            for command in &commands.commands {
                if let Command::Fan {
                    texture: Some(note),
                    ..
                } = command
                    && up.insert(note.pixels)
                {
                    uploads.push((*note, picture(note, numbers.below(2) == 0)));
                }
            }
            jobs.push(Job::Draw(Part {
                commands,
                uploads,
                written: Vec::new(),
                quads: Taken::default(),
                background: None,
                size: (320, 240),
                scale: 100,
            }));
            if numbers.below(2) == 0 {
                jobs.push(Job::ShowTarget);
            }
        }
        jobs
    }

    /// The batches as the GL's thread made them before the batching thread: its texture cache
    /// taking in the part's uploads with their opacity, then `of_layered` asking it.
    fn as_the_gl_made_them(jobs: &[Job<Part>]) -> Vec<Batches> {
        let mut cache: TextureMap<bool> = TextureMap::default();
        let mut made = Vec::new();
        for job in jobs {
            match job {
                Job::Forget(gone) => {
                    for pixels in gone {
                        cache.remove(pixels);
                    }
                }
                Job::Draw(part) => {
                    for (note, bytes) in &part.uploads {
                        let opaque = bytes.as_chunks::<4>().0.iter().all(|p| p[3] == 0xFF);
                        cache.insert(note.pixels, opaque);
                    }
                    let opaque = |note: &TextureNote| cache.get(&note.pixels).is_some_and(|o| *o);
                    made.push(Batches::of_layered(
                        &part.commands,
                        part.size.0 as u32,
                        part.size.1 as u32,
                        &opaque,
                        same_size,
                        fast_path_wanted(),
                    ));
                }
                _ => {}
            }
        }
        made
    }

    fn assert_same(a: &Batches, b: &Batches, part: usize) {
        assert_eq!(a.vertices, b.vertices, "part {part}: vertices");
        assert_eq!(a.batches, b.batches, "part {part}: batches");
        assert_eq!(a.seconds, b.seconds, "part {part}: second passes");
        assert_eq!(a.vertex_textures, b.vertex_textures, "part {part}: layers");
        let ids = |x: &Batches| -> Vec<_> {
            x.textures
                .iter()
                .map(|t| (t.pixels, t.width, t.height))
                .collect()
        };
        assert_eq!(ids(a), ids(b), "part {part}: textures");
    }

    #[test]
    fn emptied_commands_and_reused_batches_change_no_batch() {
        for seed in 1..=6 {
            let made_before = as_the_gl_made_them(&stream(seed));
            let mut opacity = Opacity::default();
            let mut n = 0;
            for job in stream(seed) {
                let Job::Draw(b) = batched(&mut opacity, job, same_size, false) else {
                    continue;
                };
                assert!(b.part.commands.is_empty(), "part {n}: commands kept");
                assert_same(&made_before[n], &b.batches, n);
                // Handed back and made again over the same room, as the GL does.
                super::super::batches::give_back_batches(b.batches);
                n += 1;
            }
            assert_eq!(n, made_before.len());
        }
    }

    #[test]
    fn batches_made_beside_the_gl_are_the_ones_it_made() {
        let (mut parts, mut seconds, mut layered) = (0, 0, 0);
        for seed in 1..=12 {
            let made_before = as_the_gl_made_them(&stream(seed));
            let mut opacity = Opacity::default();
            let made_now: Vec<Batched> = stream(seed)
                .into_iter()
                .filter_map(|job| match batched(&mut opacity, job, same_size, true) {
                    Job::Draw(b) => Some(b),
                    _ => None,
                })
                .collect();
            assert_eq!(made_before.len(), made_now.len());
            for (n, (before, now)) in made_before.iter().zip(&made_now).enumerate() {
                assert_same(before, &now.batches, n);
                assert_eq!(now.opaque.len(), now.batches.textures.len());
                parts += 1;
                seconds += now
                    .batches
                    .batches
                    .iter()
                    .filter(|b| b.second.is_some())
                    .count();
                layered += now
                    .batches
                    .vertex_textures
                    .iter()
                    .filter(|t| t[0] != NO_TEXTURE)
                    .count();
            }
        }
        // The streams reach what depends on the cache's facts: second passes joined (over an
        // opaque first only) and textured vertices.
        assert!(
            parts > 400 && seconds > 100 && layered > 1000,
            "{parts} {seconds} {layered}"
        );
    }

    #[test]
    fn the_other_jobs_and_a_parts_rows_pass_as_they_came() {
        let mut opacity = Opacity::default();
        let a = note(1, 8);
        let part = Part {
            commands: CommandList::new(),
            uploads: vec![(a, picture(&a, true))],
            written: vec![Written {
                first: 3,
                last: 5,
                bytes: vec![7; 12],
            }],
            quads: Taken::default(),
            background: Some(Background::Held),
            size: (4, 8),
            scale: 100,
        };
        let Job::Draw(b) = batched(&mut opacity, Job::Draw(part), same_size, true) else {
            panic!("a part not drawn");
        };
        assert!(b.quad_vertices.is_empty() && matches!(b.part.background, Some(Background::Held)));
        let w = &b.part.written[0];
        assert_eq!((w.first, w.last, &w.bytes[..]), (3, 5, &[7; 12][..]));
        assert_eq!((b.part.size, b.part.uploads.len()), ((4, 8), 1));
        assert!(b.batches.batches.is_empty() && b.opaque.is_empty() && opacity.of(&a));
        let job = Job::ShowPicture {
            frame: vec![1, 2],
            width: 1,
            height: 2,
        };
        let Job::ShowPicture {
            frame,
            width,
            height,
        } = batched(&mut opacity, job, same_size, true)
        else {
            panic!("a picture not shown");
        };
        assert_eq!((frame, width, height), (vec![1, 2], 1, 2));
        let (answer, picture) = std::sync::mpsc::channel();
        let Job::ReadTarget(answer) =
            batched(&mut opacity, Job::ReadTarget(answer), same_size, true)
        else {
            panic!("a read lost");
        };
        answer.send(vec![9]).unwrap();
        assert_eq!(picture.recv().unwrap(), vec![9]);
        assert!(matches!(
            batched(&mut opacity, Job::ShowTarget, same_size, true),
            Job::ShowTarget
        ));
    }

    #[test]
    fn a_parts_quads_come_out_as_vertices_with_its_background_and_texels_kept() {
        let rect = quads::Rect {
            x: 1,
            y: 2,
            width: 3,
            height: 4,
            u: 5,
            v: 6,
            palette: None,
            solid: Some([1, 2, 3, 255]),
        };
        let part = Part {
            commands: CommandList::new(),
            uploads: Vec::new(),
            written: Vec::new(),
            quads: Taken {
                rects: vec![rect, rect],
                uploads: Vec::new(),
                size: (8, 8),
            },
            background: Some(Background::Upload(vec![9; 4])),
            size: (8, 8),
            scale: 100,
        };
        let Job::Draw(b) = batched(&mut Opacity::default(), Job::Draw(part), same_size, true)
        else {
            panic!("a part not drawn");
        };
        assert_eq!(b.quad_vertices, quads::vertices(&[rect, rect]));
        assert_eq!(b.quad_vertices.len(), 12);
        assert!(matches!(&b.part.background, Some(Background::Upload(bytes)) if bytes == &[9; 4]));
    }

    #[test]
    fn a_texture_written_over_takes_its_new_opacity_and_one_gone_has_none() {
        let mut opacity = Opacity::default();
        let a = note(1, 8);
        opacity.brought_up(&a, &picture(&a, true));
        assert!(opacity.of(&a));
        opacity.brought_up(&a, &picture(&a, false));
        assert!(!opacity.of(&a));
        opacity.brought_up(&a, &picture(&a, true));
        opacity.forget(&[a.pixels]);
        assert!(!opacity.of(&a));
        assert!(!opacity.of(&note(2, 8)));
    }
}

#[cfg(test)]
mod dump_breaks {
    //! A recorded frame (`RC2K_DUMP`, the file named by `RC2K_BREAKS_DUMP`) batched as the
    //! handheld batches it, and what still splits its batches: how many neighbouring batches
    //! differ in each thing a draw sets. Skips without the file (it holds the game's textures, so
    //! it is not in the repository).

    use super::super::batches::{Batch, Batches};
    use super::super::{CommandList, STATES, TextureNote};

    struct Reader<'a>(&'a [u8]);
    impl Reader<'_> {
        fn dword(&mut self) -> u32 {
            let (head, rest) = self.0.split_first_chunk::<4>().unwrap();
            self.0 = rest;
            u32::from_le_bytes(*head)
        }
    }

    /// As `gles::same_array`.
    fn same_array(a: &TextureNote, b: &TextureNote) -> bool {
        const MAIN_SIZES: [(usize, usize); 2] = [(128, 128), (256, 256)];
        let (a, b) = ((a.width, a.height), (b.width, b.height));
        a == b || (MAIN_SIZES.contains(&a) && MAIN_SIZES.contains(&b))
    }

    #[test]
    fn what_splits_a_recorded_frames_batches() {
        let Some(path) = std::env::var_os("RC2K_BREAKS_DUMP") else {
            println!("SKIPPED: no RC2K_BREAKS_DUMP");
            return;
        };
        let file = std::fs::read(path).unwrap();
        assert_eq!(&file[..8], b"RC2KDUMP");
        let mut r = Reader(&file[8..]);
        let (width, height, _bits) = (r.dword(), r.dword(), r.dword());
        let mut notes = Vec::new();
        for _ in 0..r.dword() {
            let [w, h, pitch, each] = [0; 4].map(|_: u8| r.dword() as usize);
            let masks = [0; 4].map(|_: u8| r.dword());
            let len = r.dword() as usize;
            let (data, rest) = r.0.split_at(len);
            r.0 = rest;
            notes.push(TextureNote {
                pixels: crate::platform::direct3d::test_level(data.to_vec()),
                len,
                width: w,
                height: h,
                pitch,
                bytes: each,
                masks,
            });
        }
        let opaque_of: Vec<bool> = notes
            .iter()
            .map(|n| super::opaque_picture(&super::super::texture_as_bgra(n)))
            .collect();
        let mut list = CommandList::new();
        let mut shared: Option<std::sync::Arc<[u32; STATES]>> = None;
        for _ in 0..r.dword() {
            if r.dword() == 1 {
                let corners = r.dword() as usize;
                let vertices: Vec<[u32; 8]> = (0..corners)
                    .map(|_| [0; 8].map(|_: u8| r.dword()))
                    .collect();
                let states = [0; STATES].map(|_: u8| r.dword());
                if shared.as_deref() != Some(&states) {
                    shared = Some(std::sync::Arc::new(states));
                }
                let number = r.dword();
                let texture = (number != u32::MAX).then(|| notes[number as usize]);
                list.push_fan(
                    &vertices,
                    shared.clone().unwrap(),
                    texture,
                    (f32::MIN, f32::MIN, f32::MAX, f32::MAX),
                );
            } else {
                list.push_clear([0; 4].map(|_: u8| r.dword() as usize));
            }
        }
        let opaque = |note: &TextureNote| {
            notes
                .iter()
                .position(|n| n.pixels == note.pixels)
                .is_some_and(|n| opaque_of[n])
        };
        // The batches' fingerprints, unlayered (the PC's GPU), with second passes, and layered.
        {
            use std::hash::{Hash, Hasher};
            let print = |name: &str, b: &Batches| {
                let mut h = std::collections::hash_map::DefaultHasher::new();
                b.vertices.hash(&mut h);
                format!("{:?}", b.batches).hash(&mut h);
                b.seconds.hash(&mut h);
                b.vertex_textures.hash(&mut h);
                for t in &b.textures {
                    (t.pixels, t.width, t.height).hash(&mut h);
                }
                println!(
                    "FINGERPRINT {name}: {} batches {:016x}",
                    b.batches.len(),
                    h.finish()
                );
            };
            print("plain", &Batches::of(&list, width, height, None));
            print("seconds", &Batches::of(&list, width, height, Some(&opaque)));
            print(
                "layered",
                &Batches::of_layered(&list, width, height, &opaque, same_array, false),
            );
            let fast = Batches::of_layered(&list, width, height, &opaque, same_array, true);
            let switches = fast
                .batches
                .windows(2)
                .filter(|p| p[0].discard_free != p[1].discard_free)
                .count();
            let area = |keep: &dyn Fn(&Batch) -> bool| -> f64 {
                let size = (f64::from(width), f64::from(height));
                fast.batches
                    .iter()
                    .filter(|b| keep(b))
                    .map(|b| {
                        let at = |v: &[u32; 8]| {
                            (
                                f64::from(f32::from_bits(v[0])),
                                f64::from(f32::from_bits(v[1])),
                            )
                        };
                        fast.vertices
                            [b.first_vertex as usize..(b.first_vertex + b.vertices) as usize]
                            .as_chunks::<3>()
                            .0
                            .iter()
                            .map(|t| {
                                super::super::seethru::triangle_pixels(t.each_ref().map(at), size)
                            })
                            .sum::<f64>()
                    })
                    .sum::<f64>()
                    / (size.0 * size.1)
            };
            println!(
                "FAST PATH screens: all {:.2}, no alpha test {:.2}, blend off {:.2}, second passes {:.2}, blended still {:.2}",
                area(&|_| true),
                area(&|b| b.discard_free),
                area(&|b| b.pipeline.blend.is_none() && b.second.is_none() && b.scissor.is_none()),
                area(&|b| b.second.is_some()),
                area(&|b| b.pipeline.blend.is_some()),
            );
            println!(
                "FAST PATH: {} batches (without: {}), {} with no alpha test, {} of them with the blend off, {} with the blend off in all, {} program switches",
                fast.batches.len(),
                Batches::of_layered(&list, width, height, &opaque, same_array, false)
                    .batches
                    .len(),
                fast.batches.iter().filter(|b| b.discard_free).count(),
                fast.batches
                    .iter()
                    .filter(|b| b.discard_free && b.pipeline.blend.is_none())
                    .count(),
                fast.batches
                    .iter()
                    .filter(|b| b.pipeline.blend.is_none())
                    .count(),
                switches,
            );
        }
        let start = std::time::Instant::now();
        let batches = Batches::of_layered(&list, width, height, &opaque, same_array, true);
        let took = start.elapsed();
        let changes = |f: &dyn Fn(&Batch, &Batch) -> bool| {
            batches
                .batches
                .windows(2)
                .filter(|p| f(&p[0], &p[1]))
                .count()
        };
        println!(
            "{} commands, {} batches ({took:?}), {} textures; neighbours differing in: values {}, a second pass's values {}, second pass or not {}, sampler {}, depth write {}, blend {}, depth test or culling {}",
            list.len(),
            batches.batches.len(),
            batches.textures.len(),
            changes(&|a, b| a.values != b.values),
            changes(&|a, b| a.second.map(|s| s.values) != b.second.map(|s| s.values)),
            changes(&|a, b| a.second.is_some() != b.second.is_some()),
            changes(&|a, b| (a.linear, a.clamp) != (b.linear, b.clamp)),
            changes(&|a, b| a.pipeline.depth_write != b.pipeline.depth_write),
            changes(&|a, b| a.pipeline.blend != b.pipeline.blend),
            changes(&|a, b| {
                (a.pipeline.depth_function, a.pipeline.cull)
                    != (b.pipeline.depth_function, b.pipeline.cull)
            }),
        );
    }
}
