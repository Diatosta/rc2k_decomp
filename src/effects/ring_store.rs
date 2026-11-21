//! The ring's work and the surface pictures' names.

use crate::memory::owned::Owned;
use crate::memory::plain_buffer::PlainBuffer;

/// The ring's work: its list, cursor and count, the emitters' and the draws' integers.
pub struct RingWork {
    /// The ring's memory list (`memory::List`, 16 bytes: `effects::set_up_surface` takes the
    /// 0x25000-byte ring on it).
    pub list: crate::memory::MemoryList,
    /// The ring's next entry to take (an entry number + 1, 0 none; see `ring_entries`).
    pub cursor: u32,
    /// The ring's first entry (an entry number + 1, as the entries' links).
    pub block: u32,
    /// An entry's `place[1]` against it ends the box kinds' draw (`effects::run_ring`).
    pub top: f32,
    /// `effects::emit_rolling_dust`'s fade (the strength times a factor).
    pub fade: f32,
    /// `effects::emit_rolling_dust`'s life.
    pub life: u32,
    /// The box's |x| and |z| (`effects::run_ring`, rounded).
    pub extent: [u32; 2],
    /// `effects::emit_wheel_grit_bursts`' direction.
    pub direction: [f32; 3],
    /// `effects::emit_wheel_grit_bursts`' two halves' strengths.
    pub strength: [f32; 2],
    /// The emitters' rate (`effects::emitters::gate`).
    pub rate: u32,
    /// The entry's way dotted with the object's (`effects::run_ring`).
    pub facing: f32,
    /// The place's length less a constant (`effects::run_ring`).
    pub near: f32,
    /// The work integer of the emitters and the ring's draws (a strength, a count, a chance, a
    /// size, a depth, an alpha).
    pub whole: u32,
    /// The entries taken.
    pub count: u32,
    /// The place less the object's (`effects::run_ring`).
    pub away: [f32; 3],
    /// The same turned by the object's matrix.
    pub turned: [f32; 3],
}

impl RingWork {
    pub const fn new() -> Self {
        RingWork {
            list: crate::memory::MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            },
            cursor: 0,
            block: 0,
            top: -32768.0,
            fade: 0.0,
            life: 0,
            extent: [0; 2],
            direction: [0.0; 3],
            strength: [0.0; 2],
            rate: 0,
            facing: 0.0,
            near: 0.0,
            whole: 0,
            count: 0,
            away: [0.0; 3],
            turned: [0.0; 3],
        }
    }
}

/// The list is the one the work holds.
impl crate::memory::HeldList for std::cell::RefCell<RingWork> {
    fn get(&self) -> crate::memory::MemoryList {
        self.borrow().list
    }
    fn set(&self, list: crate::memory::MemoryList) {
        self.borrow_mut().list = list;
    }
}

/// The surface pictures' names: `effects::set_up_surface` reads them.
pub struct PictureNames {
    /// "files\m\surfdata.pcx".
    pub surfdata: PlainBuffer<21>,
    /// "files\m\surfmask.pcx".
    pub surfmask: PlainBuffer<21>,
}

impl PictureNames {
    pub const fn new() -> Self {
        PictureNames {
            surfdata: PlainBuffer::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x75, 0x72, 0x66, 0x64, 0x61,
                0x74, 0x61, 0x2E, 0x70, 0x63, 0x78, 0x00,
            ]),
            surfmask: PlainBuffer::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x75, 0x72, 0x66, 0x6D, 0x61,
                0x73, 0x6B, 0x2E, 0x70, 0x63, 0x78, 0x00,
            ]),
        }
    }
}

/// The marks ring's state, one part of the effects' owner (`effects::owner::Effects`): the ring's
/// entries, the particle pool, the ring work, the picture names, the kinds' tuning and the
/// 257 colours.
pub struct Ring {
    pub entries: Owned<Vec<Entry>>,
    pub pool: RingPool,
    pub work: std::cell::RefCell<RingWork>,
    pub names: PictureNames,
    pub kinds: RingKinds,
    pub colours: Colours257,
}

impl Ring {
    pub const fn new() -> Self {
        Ring {
            entries: Owned::new("no marks ring"),
            pool: RingPool::new(),
            work: std::cell::RefCell::new(RingWork::new()),
            names: PictureNames::new(),
            kinds: RingKinds::new(),
            colours: Colours257::new(),
        }
    }
}

/// An entry's bytes.
#[cfg(test)]
pub const ENTRY_LEN: u32 = 0x94;

/// One ring entry: 37 dwords (the size, alpha and pull fields stay dwords: kinds differ in
/// whether they hold a float or a whole number). The three-axis fields are arrays x, y, z.
/// Fields no code reads or writes by name are `reserved_<offset>`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Entry {
    /// The link to the entry before it in the ring (an entry number + 1).
    pub prev: u32,
    /// The link to the entry after it.
    pub next: u32,
    /// The emitter's tag (`EMITTER.tag`) at the time it was taken.
    pub emitter_tag: u32,
    /// The particle's kind (1, 5, 6, 0xB, 0xD, 0x10 ..).
    pub kind: u32,
    /// Its place.
    pub place: [f32; 3],
    /// Its last place (the streaks), kept for the next frame's segment.
    pub last_place: [f32; 3],
    /// The height of the ground under it: it dies when its y is below this.
    pub ground_y: f32,
    /// Its motion a frame: the place plus the motion times the ticks is how it moves.
    pub motion: [f32; 3],
    /// The burst particles' extra push (three floats; the car's `motion` times
    /// minus an eighth, x first).
    pub push: [f32; 3],
    /// Three dwords no emitter sets (0 from the emitter that clears them).
    pub spare: [u32; 3],
    /// The sprite's angle.
    pub angle: u32,
    /// How fast the angle turns.
    pub angle_rate: u32,
    /// The sprite's cell in its picture.
    pub cell: u32,
    /// How fast the sprite's cell advances with its life (read by the draw of kinds
    /// 0x10, 0x11; no emitter writes it, so it is 0 as the ring takes it).
    pub cell_rate: f32,
    /// The sprite's size.
    pub size: u32,
    /// Kind 1's size factor (a float): the size grows by it times `k1_size`; pulled
    /// toward 65536 while the entry is near the car and faces it.
    pub size_factor: f32,
    /// The car it was emitted for (a `effects_settings_store::car_handle`).
    pub car: u32,
    /// The emitting car's `pace`, kept by the emitters of kinds 6, 0xD, 0x15.
    pub car_pace: u32,
    /// The size it is pulled toward.
    pub size_goal: u32,
    /// The sprite's alpha.
    pub alpha: u32,
    /// How much of the alpha goes a tick.
    pub alpha_fade: f32,
    /// The limits the object's box extents are tested against (kinds 6, 0x15, 0x10,
    /// 0x11); no emitter writes them, so they are 0 as the ring takes it.
    pub box_limit: [u32; 2],
    /// The ticks of life left.
    pub life: u32,
    /// No emitter writes it and no ring step reads it (only the tests' word copies touch it);
    /// an unused word.
    pub reserved_88: u32,
    /// As `reserved_88`, an unused word.
    pub reserved_8c: u32,
    /// The emitting car's tag (its `tag`, 24 bits), for the draw lists.
    pub car_tag: u32,
}

#[cfg(test)]
impl Entry {
    /// The entry's dwords in order of offset.
    pub fn words(&self) -> [u32; (ENTRY_LEN / 4) as usize] {
        let mut w = [0; (ENTRY_LEN / 4) as usize];
        let mut n = 0;
        let mut push = |v: &[u32]| {
            w[n..n + v.len()].copy_from_slice(v);
            n += v.len();
        };
        push(&[self.prev, self.next, self.emitter_tag, self.kind]);
        push(&self.place.map(f32::to_bits));
        push(&self.last_place.map(f32::to_bits));
        push(&[self.ground_y.to_bits()]);
        push(&self.motion.map(f32::to_bits));
        push(&self.push.map(f32::to_bits));
        push(&self.spare);
        push(&[
            self.angle,
            self.angle_rate,
            self.cell,
            self.cell_rate.to_bits(),
            self.size,
            self.size_factor.to_bits(),
            self.car,
            self.car_pace,
            self.size_goal,
            self.alpha,
            self.alpha_fade.to_bits(),
        ]);
        push(&self.box_limit);
        push(&[self.life, self.reserved_88, self.reserved_8c, self.car_tag]);
        assert_eq!(n, w.len());
        w
    }

    /// An entry from its dwords in order of offset.
    pub fn from_words(w: [u32; (ENTRY_LEN / 4) as usize]) -> Entry {
        Entry {
            prev: w[0],
            next: w[1],
            emitter_tag: w[2],
            kind: w[3],
            place: [w[4], w[5], w[6]].map(f32::from_bits),
            last_place: [w[7], w[8], w[9]].map(f32::from_bits),
            ground_y: f32::from_bits(w[10]),
            motion: [w[11], w[12], w[13]].map(f32::from_bits),
            push: [w[14], w[15], w[16]].map(f32::from_bits),
            spare: [w[17], w[18], w[19]],
            angle: w[20],
            angle_rate: w[21],
            cell: w[22],
            cell_rate: f32::from_bits(w[23]),
            size: w[24],
            size_factor: f32::from_bits(w[25]),
            car: w[26],
            car_pace: w[27],
            size_goal: w[28],
            alpha: w[29],
            alpha_fade: f32::from_bits(w[30]),
            box_limit: [w[31], w[32]],
            life: w[33],
            reserved_88: w[34],
            reserved_8c: w[35],
            car_tag: w[36],
        }
    }
}

/// The particle ring's pool and the burst code's scratch.
pub struct RingPool {
    /// A `memory::List` (head, total, cursor, remaining).
    pub list: crate::memory::PlainList,
    /// The first entry of the ring in use.
    pub free: core::cell::Cell<u32>,
    /// The block taken for the ring.
    pub ring: core::cell::Cell<u32>,
    /// A developer's toggle (the debug page's "fworks run"). Nothing reads or writes it.
    pub fireworks_run: core::cell::Cell<u32>,
    /// A burst particle's kind plus 8 bits.
    pub nine: core::cell::Cell<u32>,
    /// A burst particle's kind times 0x400.
    pub kind: core::cell::Cell<u32>,
    /// The burst's radius.
    pub burst_radius: core::cell::Cell<f32>,
    /// The colour index of the particle being drawn.
    pub index: core::cell::Cell<u32>,
    /// The entries taken.
    pub used: core::cell::Cell<u32>,
}

/// RingPool's first values.
impl RingPool {
    pub const fn new() -> Self {
        RingPool {
            list: crate::memory::PlainList::empty(),
            free: core::cell::Cell::new(0x0),
            ring: core::cell::Cell::new(0x0),
            fireworks_run: core::cell::Cell::new(0x0),
            nine: core::cell::Cell::new(0x0),
            kind: core::cell::Cell::new(0x0),
            burst_radius: core::cell::Cell::new(0.0),
            index: core::cell::Cell::new(0x0),
            used: core::cell::Cell::new(0x0),
        }
    }
}

/// The colour table used while the kind floats' `other` is 0: 257 colours (the surface's own
/// 128 colours are the ones in use while it is not).
pub struct Colours257 {
    /// The 257 colours
    pub colours: [u32; 257],
}

/// The 257 colours as the image has them: 0x00FFFF00 + 2 i for the first 128, then 0x00FFFFFF.
impl Colours257 {
    pub const fn new() -> Self {
        Colours257 {
            colours: {
                let mut colours = [0; 257];
                let mut i = 0;
                while i < 257 {
                    colours[i] = if i < 128 {
                        0x00FF_FF00 + 2 * i as u32
                    } else {
                        0x00FF_FFFF
                    };
                    i += 1;
                }
                colours
            },
        }
    }
}

/// The ring kinds' constants (`run_ring`, and the ribbons' two).
#[derive(Clone, Copy)]
pub struct RingKinds {
    /// Kind 5's drift factors
    pub drift5: [f32; 3],
    /// Kind 0xB's drift factors
    pub drift_b: [f32; 3],
    /// Kind 6's three dampings
    pub k6_damp: [f32; 3],
    /// Kind 6's size rate
    pub k6_pull: f32,
    /// Kind 0x15's three dampings
    pub k15_damp: [f32; 3],
    /// Kind 0x15's size rate
    pub k15_pull: f32,
    /// Kind 1's size factor (`size_factor` times it)
    pub k1_size: f32,
    /// Kind 1's three dampings
    pub k1_damp: [f32; 3],
    /// Kind 1's size rate
    pub k1_pull: f32,
    /// Less this from the length to the object (`pull_length`)
    pub near_less: f32,
    /// The strength's pull scale (`pull_strength`)
    pub near_scale: f32,
    /// The strength's pull rate (`pull_strength`)
    pub near_rate: f32,
    /// Kind 0xD's y gain
    pub kd_gain: f32,
    /// Kind 0xD's three dampings
    pub kd_damp: [f32; 3],
    /// Kind 0xD's size rate
    pub kd_pull: f32,
    /// Kind 0x13's three dampings
    pub k13_damp: [f32; 3],
    /// Kind 0x13's y gain
    pub k13_gain: f32,
    /// Kind 0x16's three dampings
    pub k16_damp: [f32; 3],
    /// Kind 0x16's y gain
    pub k16_gain: f32,
    /// Kind 0x10's y gain
    pub k10_gain: f32,
    /// Kind 0x11's y gain
    pub k11_gain: f32,
    /// A ribbon's y gain (`run_ribbons::first_frame`)
    pub ribbon_a: f32,
    /// A ribbon's y gain per frame (`run_ribbons::move_`)
    pub ribbon_b: f32,
}

/// RingKinds's first values.
impl RingKinds {
    pub const fn new() -> Self {
        RingKinds {
            drift5: [-2.0, 92.0, f32::from_bits(0xFF800000)],
            drift_b: [-2.0, 92.0, f32::from_bits(0xFF800000)],
            k6_damp: [0.001, 0.001, 0.001],
            k6_pull: 0.01_f32,
            k15_damp: [0.001, 0.001, 0.001],
            k15_pull: 0.01_f32,
            k1_size: 1.0_f32,
            k1_damp: [0.003, 0.003, 0.003],
            k1_pull: 0.0015_f32,
            near_less: 111072.0_f32,
            near_scale: 0.0002_f32,
            near_rate: 0.005_f32,
            kd_gain: 0.01_f32,
            kd_damp: [0.004, 0.004, 0.004],
            kd_pull: 0.01_f32,
            k13_damp: [0.003, 0.003, 0.003],
            k13_gain: 16.0_f32,
            k16_damp: [0.003, 0.003, 0.003],
            k16_gain: 16.0_f32,
            k10_gain: -0.5_f32,
            k11_gain: -0.5_f32,
            ribbon_a: -0.5_f32,
            ribbon_b: -0.5_f32,
        }
    }
}
