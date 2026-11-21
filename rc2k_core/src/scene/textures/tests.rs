use super::*;

#[test]
fn small() {
    assert_eq!(
        [0, 0x7F, 0x80, 0xFF, 0x100, 0x13F, 0x140, u32::MAX].map(no_mips_for),
        [false, false, true, true, false, false, true, true]
    );
    let mut loaded = [1u32; 4];
    assert!(forget_slot_texture(&mut loaded, 4).is_err());
    assert_eq!(forget_slot_texture(&mut loaded, 2), Ok(()));
    assert_eq!(loaded, [1, 1, NONE, 1]);
}

#[test]
fn needs() {
    let tags = [1, 2, 1, 1];
    let (mut system, mut video) = ([0, 0, NONE, 5], [3, 0, 1, 1]);
    let mut n = Needs {
        tags: &tags,
        system: &mut system,
        video: &mut video,
    };
    assert!(n.raise(2, 1));
    assert!(!n.raise(2, 1));
    assert_eq!((system, video), ([2, 0, NONE, 5], [3, 0, 2, 2]));
    let sizes = |l: usize| 1 << l;
    let (t, last) = count_needs(&system, &video, sizes).unwrap();
    assert_eq!(t.to_array(), [8 + 1 + 4 + 4, 4 + 32 + 1, 4, 3]);
    assert_eq!(last, 4);
    assert!(count_needs(&[LEVELS], &[0], sizes).is_err());
    assert_eq!(count_needs(&[0], &[NONE], sizes).unwrap().1, NONE);
}

#[test]
fn runs() {
    let (mut tags, mut system, mut video) = ([0u32; 0x400], [0u32; 0x400], [0u32; 0x400]);
    let mut t = LevelTables {
        tags: &mut tags,
        system: &mut system,
        video: &mut video,
    };
    assert!(t.set_run(5, 0, 1, 1).is_err());
    assert!(t.set_run(0x3FF, 2, 1, 1).is_err());
    assert_eq!(t.set_run(0x3FE, 2, 3, 0x63), Ok(()));
    assert_eq!(
        (tags[0x3FD], tags[0x3FE], system[0x3FF], video[0x3FE]),
        (0, 0x63, 3, 3)
    );
}

#[test]
fn places() {
    let mut wanted = vec![NONE; SLOTS as usize];
    let mut place_of = vec![NONE; SLOTS as usize];
    wanted[3] = 7;
    wanted[9] = 7;
    wanted[0] = 8;
    place_of[7] = 0x20;
    place_of[8] = PLACES;
    let mut places = vec![0u32; PLACES as usize];
    assert_eq!(map_places(&wanted, &place_of, &mut places), PLACES);
    assert_eq!(places[0x20], 3);
    assert_eq!(places.iter().filter(|&&p| p != NONE).count(), 1);
}

#[test]
fn assigns() {
    let mut wanted = [7u32; 4];
    assert!(assign_kind_textures(&[NONE, 4], &[1, 2], &mut wanted).is_err());
    assert_eq!(wanted, [7; 4]);
    assign_kind_textures(&[2, NONE, 2, 0], &[10, 11, 12, 13], &mut wanted).unwrap();
    assert_eq!(wanted, [13, NONE, 10, NONE]);
    let own = [0, NONE, 2, NONE, NONE, NONE, NONE, 1];
    assert_eq!(
        menu_textures(own, 0x3F0, 5),
        Ok(vec![(0x3F0, 5), (0x3F2, 7), (0x3F7, 6)])
    );
    assert!(menu_textures(own, 0x3F9, 5).is_err());
    assert!(menu_textures([NONE; 8], u32::MAX, 5).unwrap().is_empty());
    assert_eq!(
        [
            0,
            899,
            900,
            901,
            902,
            999,
            1000,
            1999,
            2000,
            2999,
            3000,
            u32::MAX
        ]
        .map(picture_folder),
        [
            PictureFolder::A,
            PictureFolder::A,
            PictureFolder::M,
            PictureFolder::M,
            PictureFolder::A,
            PictureFolder::A,
            PictureFolder::B,
            PictureFolder::B,
            PictureFolder::Front,
            PictureFolder::Front,
            PictureFolder::Extra,
            PictureFolder::Extra
        ]
    );
}

#[test]
fn look_ups() {
    let l = |w| look_up(w, |p| p as u32 + 1, |k| k as u32 * 2, || 0x10);
    assert_eq!(
        l(NONE),
        LookUp {
            texture: NO_TEXTURE,
            last_number: NONE
        }
    );
    assert_eq!(
        l(0x8000_0003),
        LookUp {
            texture: 4,
            last_number: 4
        }
    );
    assert_eq!(l(0x8000_0FA0).last_number, 0xFA0);
    assert_eq!(l(0x400).texture, NO_TEXTURE);
    assert_eq!(
        l(7),
        LookUp {
            texture: 14,
            last_number: 14
        }
    );
    assert_eq!(
        l(8),
        LookUp {
            texture: NO_TEXTURE,
            last_number: 16
        }
    );
    assert_eq!(look_up(1, |_| 0, |_| 0, || 0).texture, NO_TEXTURE);
}

struct Memory {
    levels: [Vec<u32>; 2],
    slots: Vec<(u32, u32)>,
}

impl TextureMemory for Memory {
    fn level(&self, video: bool, texture: u32) -> u32 {
        self.levels[!video as usize][texture as usize]
    }
    fn size(&self, level: u32) -> u32 {
        level * 10
    }
    fn mips(&self) -> u32 {
        1
    }
    fn level_mips(&self, level: u32) -> u32 {
        level + 100
    }
    fn forget_kinds(&mut self) {}
    fn set_first_slot(&mut self, texture: u32, slot: u32) {
        self.slots.push((texture, slot));
    }
    fn set_first_texture(&mut self, texture: u32, first: u32) -> Result<(), Invalid> {
        self.slots.push((texture | 0x8000, first));
        Ok(())
    }
}

#[test]
fn allocations() {
    let mut m = Memory {
        levels: [vec![NONE; SLOTS as usize], vec![NONE; SLOTS as usize]],
        slots: Vec::new(),
    };
    m.levels[0][0x90] = 9;
    m.levels[1][0xDC] = 2;
    let mut asks = Vec::new();
    let mut call = |c: Call| {
        asks.push(c);
        Some(5)
    };
    assert_eq!(try_allocate(&mut m, &mut call), Ok(true));
    assert_eq!(asks[1], Call::Number(90));
    assert_eq!(asks[3], Call::Number(20));
    assert_eq!(m.slots, [(0x90, 5), (0x80DC, 5)]);
    assert_eq!(
        asks[4],
        Call::Allocate(Ask {
            texture: 0x90,
            video: true,
            level: 7,
            pixel_kind: 2,
            mips: 0
        })
    );
    assert_eq!(
        asks[5],
        Call::Allocate(Ask {
            texture: 0xDC,
            video: false,
            level: 2,
            pixel_kind: 3,
            mips: 102
        })
    );
    m.levels[1][0xDC] = LEVELS;
    assert!(try_allocate(&mut m, &mut |_| None).is_err());
}

#[test]
fn slot_loads() {
    assert!(slots_in_use(0).is_err() && slots_in_use(SLOTS + 1).is_err());
    assert_eq!(slots_in_use(SLOTS), Ok(SLOTS));
    assert!(slot_load(SLOTS, || 0, || 0).is_err());
    assert_eq!(slot_load(1, || NONE, || unreachable!()), Ok(None));
    assert_eq!(slot_load(1, || 5, || 5), Ok(None));
    assert_eq!(slot_load(1, || 5, || 6), Ok(Some(5)));
}
