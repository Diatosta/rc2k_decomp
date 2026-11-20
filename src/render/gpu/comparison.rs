//! One recorded frame (`RC2K_DUMP`, the file named by `RC2K_COMPARE_FRAME`) through the
//! rasteriser and the GPU, and how far the pictures differ. Skips without the file (it holds
//! the game's textures, so it is not in the repository) or a GPU.

use super::*;
use crate::render::STATES;

struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn dword(&mut self) -> u32 {
        let (head, rest) = self.0.split_first_chunk::<4>().unwrap();
        self.0 = rest;
        u32::from_le_bytes(*head)
    }
}

#[test]
fn a_recorded_frame_looks_the_same() {
    let system = crate::test_support::own::system();
    let Some(path) = std::env::var_os("RC2K_COMPARE_FRAME") else {
        println!("SKIPPED: no RC2K_COMPARE_FRAME");
        return;
    };
    let file = std::fs::read(path).unwrap();
    assert_eq!(&file[..8], b"RC2KDUMP");
    let mut r = Reader(&file[8..]);
    let (width, height, bits) = (r.dword() as usize, r.dword() as usize, r.dword());
    assert_eq!(bits, 32);
    let mut bytes = Vec::new();
    let mut notes = Vec::new();
    for _ in 0..r.dword() {
        let [w, h, pitch, each] = [0; 4].map(|_: u8| r.dword() as usize);
        let masks = [0; 4].map(|_: u8| r.dword());
        let len = r.dword() as usize;
        let (data, rest) = r.0.split_at(len);
        r.0 = rest;
        bytes.push(data.to_vec());
        notes.push((w, h, pitch, each, masks, len));
    }
    let levels: Vec<_> = bytes
        .iter()
        .map(|b| crate::d3d::device::test_level(b.clone()))
        .collect();
    let note = |n: usize| {
        let (width, height, pitch, bytes, masks, len) = notes[n];
        TextureNote {
            pixels: levels[n],
            len,
            width,
            height,
            pitch,
            bytes,
            masks,
        }
    };
    let read_commands = |r: &mut Reader| {
        let mut commands = CommandList::new();
        for _ in 0..r.dword() {
            if r.dword() == 1 {
                let corners = r.dword() as usize;
                let vertices: Vec<[u32; 8]> = (0..corners)
                    .map(|_| [0; 8].map(|_: u8| r.dword()))
                    .collect();
                let mut states = Box::new([0; STATES]);
                for s in states.iter_mut() {
                    *s = r.dword();
                }
                let number = r.dword();
                commands.push_fan(
                    &vertices,
                    states.into(),
                    (number != u32::MAX).then(|| note(number as usize)),
                    (f32::MIN, f32::MIN, f32::MAX, f32::MAX),
                );
            } else {
                commands.push_clear([0; 4].map(|_: u8| r.dword() as usize));
            }
        }
        commands
    };
    let commands = read_commands(&mut Reader(r.0));

    system.session.screen.get().width.set(width as u32);
    system.session.screen.get().height.set(height as u32);
    system.session.screen.get().bits.set(bits);
    let soft_held = std::sync::Arc::new(crate::platform::pool::PoolBytes::from_vec(vec![
        0u8;
        width * height
            * 4
    ]));
    let soft_handle = 0x7000_0000;
    crate::gfx::locked_pixels::register(soft_handle, &soft_held);
    let target = Target {
        pixels: soft_handle,
        pitch: width * 4,
        width,
        height,
        bits,
    };
    crate::render::Backend::draw(
        &mut crate::render::software::Software {
            screen: (width, height),
        },
        &target,
        read_commands(&mut Reader(r.0)),
    );

    let soft = soft_held.to_vec();
    sdl_gpu::init_video();
    let Some(device) = make_device() else {
        println!("SKIPPED: no GPU device: {}", super::device::sdl_error());
        return;
    };
    let mut gpu = Gpu::new(device).unwrap();
    gpu.set_size(width as u32, height as u32).unwrap();
    let mut canvas = vec![0u8; width * height * 4];
    Gpu::draw(&mut gpu, &mut canvas, None, &commands).unwrap();
    gpu.bring_back(None, &mut canvas).unwrap();
    // The frame queued to come back later is the picture brought back at once.
    #[cfg(not(handheld))]
    {
        // The game's screen holds no written pixel at a frame's start (`bring_back` filled
        // `canvas` with the picture, whose alpha bytes would blend over itself).
        let mut beside = canvas.clone();
        mark_unwritten(&mut beside);
        let mut queued = vec![0xAA; 3];
        assert!(
            !gpu.collect_video_read(&mut queued).unwrap(),
            "nothing is queued yet"
        );
        gpu.start_video_read(None, &mut beside, false).unwrap();
        assert!(gpu.collect_video_read(&mut queued).unwrap());
        assert!(
            queued == canvas,
            "the queued picture differs from the brought-back one"
        );
        assert!(
            !gpu.collect_video_read(&mut queued).unwrap(),
            "collected twice"
        );
    }

    let (mut far, mut total) = (0usize, 0u64);
    for (a, b) in soft
        .as_chunks::<4>()
        .0
        .iter()
        .zip(canvas.as_chunks::<4>().0)
    {
        let most = (0..3).map(|n| a[n].abs_diff(b[n])).max().unwrap();
        total += u64::from(most);
        if most > 32 {
            far += 1;
        }
    }
    let pixels = width * height;
    println!(
        "{width} x {height}, {} commands: {:.2}% of pixels differ by more than 32, mean difference {:.2}",
        commands.len(),
        far as f64 * 100.0 / pixels as f64,
        total as f64 / pixels as f64
    );
    if let Some(out) = std::env::var_os("RC2K_COMPARE_OUT") {
        let out = std::path::PathBuf::from(out);
        std::fs::write(out.join("software.raw"), &soft).unwrap();
        std::fs::write(out.join("gpu.raw"), &canvas).unwrap();
    }
    assert!(far * 20 < pixels, "over 5% of the pixels differ a lot");
}
