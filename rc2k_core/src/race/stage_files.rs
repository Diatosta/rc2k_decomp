//! The stage's data files parsed: the preferences (`localdat`) and the course info (the texture
//! and mark info files), and the texture infos made from them.

use crate::Invalid;
use crate::race::stage::{self, Swap, TextureInfo};

/// The preferences' size: what the stage's `localdat` fills.
pub const PREFS_SIZE: usize = 0x206C;
/// The 256 texture kinds, as dwords.
pub const TEXTURE_KINDS_AT: usize = 0x100;
/// How many texture kinds there are.
pub const TEXTURE_KINDS: usize = 256;
/// The stage's precipitation range: the low and the high end of the noise scale the rain and snow
/// amount is read from (two dwords).
pub const PRECIPITATION_RANGE_AT: usize = 0x690;
/// The car light table's three dwords.
pub const CAR_LIGHT_TABLE_AT: usize = 0x18B0;
/// The 0x40 swapped picture kinds, as dwords.
pub const PICTURE_LISTS_B_AT: usize = 0x19C8;

/// The stage's preferences: `PREFS_SIZE` bytes, the files laid over what was there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StagePrefsFile {
    bytes: Vec<u8>,
}

impl StagePrefsFile {
    /// The preferences held now (`PREFS_SIZE` bytes).
    pub fn from_held(held: &[u8]) -> Result<Self, Invalid> {
        if held.len() != PREFS_SIZE {
            return Err(Invalid("preferences of the wrong size"));
        }
        Ok(Self {
            bytes: held.to_vec(),
        })
    }

    /// A file's first bytes laid over the start (all of them, or `PREFS_SIZE`); what the file
    /// does not reach is kept.
    pub fn overlay(&mut self, file: &[u8]) {
        let n = file.len().min(PREFS_SIZE);
        self.bytes[..n].copy_from_slice(&file[..n]);
    }

    /// The bytes, in file order.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn dword(&self, at: usize) -> u32 {
        u32::from_le_bytes(self.bytes[at..at + 4].try_into().expect("four bytes"))
    }

    fn set_dword(&mut self, at: usize, value: u32) {
        self.bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// The kind of texture `n` (below `TEXTURE_KINDS`).
    pub fn texture_kind(&self, n: usize) -> u32 {
        self.dword(TEXTURE_KINDS_AT + 4 * n)
    }

    /// The texture kinds, in order.
    pub fn texture_kinds(&self) -> [u32; TEXTURE_KINDS] {
        std::array::from_fn(|n| self.texture_kind(n))
    }

    /// Place `n` of the swapped picture list (below `stage::SWAPPED_PLACES`).
    pub fn swapped(&self, n: u32) -> u32 {
        self.dword(PICTURE_LISTS_B_AT + 4 * n as usize)
    }

    /// For the stages that swap a texture kind (`stage::TEXTURE_SWAPS`): the first of the first
    /// 0xFF kinds that is the old one becomes the new one, which goes >> 4 into the swapped
    /// list's first free place unless it is listed (the test is made before the shift: kept).
    pub fn swap_stage_texture(&mut self, stage_number: u32) -> Swap {
        let ((_, new), swap) = stage::swap_texture(
            stage_number,
            |n| self.texture_kind(n as usize),
            |n| self.swapped(n),
        );
        let kind = match swap {
            Swap::NotThisStage | Swap::NotFound => return swap,
            Swap::Listed { kind, .. } | Swap::Full { kind } => kind,
            Swap::Added { kind, at } => {
                self.set_dword(PICTURE_LISTS_B_AT + 4 * at as usize, new >> 4);
                kind
            }
        };
        self.set_dword(TEXTURE_KINDS_AT + 4 * kind as usize, new);
        swap
    }

    /// For the tweaked stages (`stage::stage_tweaked`): the car light table is 0xFF, 0xFF, 0xFF.
    pub fn tweak_stage(&mut self, stage_number: u32) {
        if stage::stage_tweaked(stage_number) {
            for k in 0..3 {
                self.set_dword(CAR_LIGHT_TABLE_AT + 4 * k, 0xFF);
            }
        }
    }

    /// Without precipitation the range is 0 .. 0: no rain or snow amount, and no screen picture for
    /// the windscreen's drops and flakes.
    pub fn clear_precipitation_range(&mut self, no_precipitation: bool) {
        if no_precipitation {
            self.set_dword(PRECIPITATION_RANGE_AT, 0);
            self.set_dword(PRECIPITATION_RANGE_AT + 4, 0);
        }
    }
}

/// The course info's size.
pub const INFO_SIZE: usize = 0x40_0000;
/// Where `files\a\textinfo` goes, and how much of it.
pub const TEXTURE_INFO_A: (usize, usize) = (0, 0x1F_4000);
/// Where `files\m\markinfo` goes, and how much of it.
pub const MARK_INFO: (usize, usize) = (0x1C_2000, 0x8000);
/// Where `files\b\textinfo` goes, and how much of it.
pub const TEXTURE_INFO_B: (usize, usize) = (0x1F_4000, 0x1F_4000);
/// Where the three line feeds go.
pub const LINE_FEEDS_AT: usize = 0x5_BD21;
/// The tail's source and its destination (0x800 bytes), copied last.
pub const TAIL: (usize, usize, usize) = (0x17_E800, 0x17_F000, 0x800);
/// The bytes of a texture kind's record in the info.
pub const KIND_RECORD: usize = 0x80;

/// The course info: `INFO_SIZE` bytes, zero where no file reached.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CourseInfo {
    bytes: Vec<u8>,
}

impl Default for CourseInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl CourseInfo {
    pub fn new() -> Self {
        Self {
            bytes: vec![0; INFO_SIZE],
        }
    }

    /// A file's first bytes (at most `part.1`) laid at `part.0`; what the file does not reach
    /// stays as it was. Refused: a part outside the info.
    pub fn lay_file(&mut self, part: (usize, usize), file: &[u8]) -> Result<(), Invalid> {
        let (at, room) = part;
        if at.checked_add(room).is_none_or(|end| end > INFO_SIZE) {
            return Err(Invalid("a file read outside the course info"));
        }
        let n = file.len().min(room);
        self.bytes[at..at + n].copy_from_slice(&file[..n]);
        Ok(())
    }

    /// The finish of the texture info: three line feeds at `LINE_FEEDS_AT`, then the tail copied
    /// (the quirks of the data files, kept).
    pub fn finish_texture_info(&mut self) {
        self.bytes[LINE_FEEDS_AT..LINE_FEEDS_AT + 3].fill(0x0A);
        let (from, to, len) = TAIL;
        self.bytes.copy_within(from..from + len, to);
    }

    /// The bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The record of texture kind `kind` (`None` past the info).
    pub fn kind_record(&self, kind: u32) -> Option<&[u8]> {
        let at = (kind as usize).checked_mul(KIND_RECORD)?;
        self.bytes.get(at..at.checked_add(KIND_RECORD)?)
    }
}

/// The 256 textures' infos from their kinds and the info's records (`record(kind)` is the
/// kind's `KIND_RECORD` bytes). Refused: a kind whose record the info does not hold.
pub fn texture_infos<R: AsRef<[u8]>>(
    kinds: &[u32; TEXTURE_KINDS],
    mut record: impl FnMut(u32) -> Option<R>,
) -> Result<Vec<TextureInfo>, Invalid> {
    let mut out = Vec::with_capacity(TEXTURE_KINDS);
    for &kind in kinds {
        let rec = record(kind).ok_or(Invalid("a texture kind past the course info"))?;
        let rec = rec.as_ref();
        if rec.len() < KIND_RECORD {
            return Err(Invalid("a texture kind's record cut short"));
        }
        let dword = |o: u32| {
            let o = o as usize;
            u32::from_le_bytes(rec[o..o + 4].try_into().expect("four bytes"))
        };
        out.push(stage::texture_info(kind, dword, |o| rec[o as usize]));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefs() -> StagePrefsFile {
        StagePrefsFile::from_held(&vec![0xAA; PREFS_SIZE]).unwrap()
    }

    #[test]
    fn prefs_overlay_keeps_what_the_file_does_not_reach() {
        let mut p = prefs();
        p.overlay(&[1, 2, 3, 4, 5]);
        assert_eq!(&p.bytes()[..6], &[1, 2, 3, 4, 5, 0xAA]);
        p.overlay(&[]);
        assert_eq!(&p.bytes()[..6], &[1, 2, 3, 4, 5, 0xAA]);
        let long = vec![7u8; PREFS_SIZE + 100];
        p.overlay(&long);
        assert_eq!(p.bytes().len(), PREFS_SIZE);
        assert!(p.bytes().iter().all(|&b| b == 7));
        assert!(StagePrefsFile::from_held(&[0; 5]).is_err());
    }

    #[test]
    fn a_swap_is_the_swap_texture_one() {
        let mut p = StagePrefsFile::from_held(&vec![0; PREFS_SIZE]).unwrap();
        // Stage 0x20 swaps 0x2F82 for 0x2FD0; kind 9 is the old one.
        p.set_dword(TEXTURE_KINDS_AT + 4 * 9, 0x2F82);
        p.set_dword(PICTURE_LISTS_B_AT, 5);
        let swap = p.swap_stage_texture(0x20);
        assert_eq!(swap, Swap::Added { kind: 9, at: 1 });
        assert_eq!(p.texture_kind(9), 0x2FD0);
        assert_eq!(p.swapped(1), 0x2FD0 >> 4);
        // Not this stage, or no old kind: untouched.
        let before = p.clone();
        assert_eq!(p.swap_stage_texture(0x10), Swap::NotThisStage);
        assert_eq!(p.swap_stage_texture(0x20), Swap::NotFound);
        assert_eq!(p, before);
    }

    #[test]
    fn a_listed_new_kind_is_not_listed_again() {
        let mut p = StagePrefsFile::from_held(&vec![0; PREFS_SIZE]).unwrap();
        p.set_dword(TEXTURE_KINDS_AT, 0x2F82);
        p.set_dword(PICTURE_LISTS_B_AT + 8, 0x2FD0);
        assert_eq!(p.swap_stage_texture(0x22), Swap::Listed { kind: 0, at: 2 });
        assert_eq!(p.texture_kind(0), 0x2FD0);
        assert_eq!(p.swapped(0), 0);
    }

    #[test]
    fn tweaks_and_clears() {
        let mut p = prefs();
        p.tweak_stage(0x14);
        assert_eq!(p, prefs());
        p.tweak_stage(0x3E);
        assert_eq!(p.dword(CAR_LIGHT_TABLE_AT + 8), 0xFF);
        assert_eq!(p.dword(CAR_LIGHT_TABLE_AT + 12), 0xAAAA_AAAA);
        p.clear_precipitation_range(false);
        assert_eq!(p.dword(PRECIPITATION_RANGE_AT), 0xAAAA_AAAA);
        p.clear_precipitation_range(true);
        assert_eq!(
            (
                p.dword(PRECIPITATION_RANGE_AT),
                p.dword(PRECIPITATION_RANGE_AT + 4)
            ),
            (0, 0)
        );
        assert_eq!(p.dword(PRECIPITATION_RANGE_AT + 8), 0xAAAA_AAAA);
    }

    #[test]
    fn info_files_lay_in_order_with_the_quirks() {
        let mut info = CourseInfo::new();
        info.lay_file(TEXTURE_INFO_A, &vec![1; 0x1F_4000 + 50])
            .unwrap();
        info.lay_file(MARK_INFO, &[2; 10]).unwrap();
        info.lay_file(TEXTURE_INFO_B, &[3; 20]).unwrap();
        info.bytes[TAIL.0 + 5] = 9;
        info.finish_texture_info();
        let b = info.bytes();
        assert_eq!(b[0x1F_3FFF], 1);
        assert_eq!(b[0x1F_4000], 3);
        assert_eq!(b[0x1F_4000 + 20], 0);
        assert_eq!(b[0x1C_2000 + 9], 2);
        assert_eq!(b[0x1C_2000 + 10], 1);
        assert_eq!(b[LINE_FEEDS_AT - 1], 1);
        assert_eq!(&b[LINE_FEEDS_AT..LINE_FEEDS_AT + 3], &[0x0A; 3]);
        assert_eq!(b[LINE_FEEDS_AT + 3], 1);
        assert_eq!(b[TAIL.1 + 5], 9);
        assert!(info.lay_file((INFO_SIZE - 4, 8), &[]).is_err());
        assert!(info.lay_file((usize::MAX, 8), &[]).is_err());
    }

    #[test]
    fn texture_infos_follow_the_kinds_and_refuse_past_the_info() {
        let mut info = CourseInfo::new();
        for (k, b) in info.bytes.iter_mut().enumerate() {
            *b = k as u8;
        }
        let mut kinds = [0u32; TEXTURE_KINDS];
        kinds[1] = 0x25;
        let all = texture_infos(&kinds, |k| info.kind_record(k)).unwrap();
        assert_eq!(all.len(), TEXTURE_KINDS);
        let rec = info.kind_record(0x25).unwrap();
        let want = stage::texture_info(
            0x25,
            |o| u32::from_le_bytes(rec[o as usize..o as usize + 4].try_into().unwrap()),
            |o| rec[o as usize],
        );
        assert_eq!(all[1], want);
        assert_eq!(all[0].row, 0);
        kinds[7] = (INFO_SIZE / KIND_RECORD) as u32;
        assert!(texture_infos(&kinds, |k| info.kind_record(k)).is_err());
        assert!(info.kind_record(u32::MAX).is_none());
        assert!(
            info.kind_record((INFO_SIZE / KIND_RECORD) as u32 - 1)
                .is_some()
        );
    }
}
