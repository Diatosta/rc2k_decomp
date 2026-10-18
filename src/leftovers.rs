//! What a self-test leaves changed in the game's writable sections (`selftest <pat> leftovers`).
//! The game is paused while a test runs, so what differs after it is the test's doing (or a
//! thread let through the pause: the log writer). For finding a test that does not put
//! something back without failing (2026-10-18: the stage selection's selectors went white
//! after `selftest screens::`). Heap memory (fonts' palettes, lists) is not covered.

use crate::framework::{IMAGE_BASE, game_addr};

const WRITABLE: u32 = 0x8000_0000; // IMAGE_SCN_MEM_WRITE
/// Changed bytes closer than this are reported as one range.
const JOIN: usize = 16;
/// Ranges logged per test.
const LOGGED: usize = 12;

/// A copy of every writable section of RAL.EXE, by IDA address.
pub struct Snapshot(Vec<(usize, Vec<u8>)>);

/// (IDA address, length) of each writable section.
fn sections() -> Vec<(usize, usize)> {
    let base = game_addr(0);
    let at = |off: usize| unsafe { (base as *const u8).add(off) };
    unsafe {
        let nt = (at(0x3C) as *const u32).read_unaligned() as usize;
        let count = (at(nt + 6) as *const u16).read_unaligned() as usize;
        let optional = (at(nt + 0x14) as *const u16).read_unaligned() as usize;
        let table = nt + 0x18 + optional;
        (0..count)
            .filter_map(|n| {
                let s = table + n * 40;
                let size = (at(s + 8) as *const u32).read_unaligned() as usize;
                let start = (at(s + 0xC) as *const u32).read_unaligned() as usize;
                let flags = (at(s + 0x24) as *const u32).read_unaligned();
                (flags & WRITABLE != 0 && size != 0).then_some((IMAGE_BASE + start, size))
            })
            .collect()
    }
}

impl Snapshot {
    pub fn take() -> Self {
        Self(
            sections()
                .into_iter()
                .map(|(ida, len)| {
                    let p = game_addr(ida - IMAGE_BASE) as *const u8;
                    (ida, unsafe { std::slice::from_raw_parts(p, len) }.to_vec())
                })
                .collect(),
        )
    }

    /// The ranges (IDA address, length, bytes before, bytes now) that differ from now.
    pub fn changed(&self) -> Vec<(usize, usize, Vec<u8>, Vec<u8>)> {
        let mut out = Vec::new();
        for (ida, before) in &self.0 {
            let p = game_addr(ida - IMAGE_BASE) as *const u8;
            let now = unsafe { std::slice::from_raw_parts(p, before.len()) };
            let mut n = 0;
            while n < before.len() {
                if before[n] == now[n] {
                    n += 1;
                    continue;
                }
                let start = n;
                let mut last = n;
                while n < before.len() && n - last < JOIN {
                    if before[n] != now[n] {
                        last = n;
                    }
                    n += 1;
                }
                let end = last + 1;
                out.push((
                    ida + start,
                    end - start,
                    before[start..end].to_vec(),
                    now[start..end].to_vec(),
                ));
            }
        }
        out
    }
}

/// Logs what `hook`'s test left changed since `before`.
pub fn report(name: &str, before: &Snapshot) {
    let changed = before.changed();
    if changed.is_empty() {
        return;
    }
    let total: usize = changed.iter().map(|c| c.1).sum();
    crate::framework::log(&format!(
        "LEFTOVER {name}: {} range(s), {total} byte(s)",
        changed.len()
    ));
    let hex = |b: &[u8]| {
        let shown: Vec<String> = b.iter().take(16).map(|x| format!("{x:02x}")).collect();
        shown.join(" ") + if b.len() > 16 { " .." } else { "" }
    };
    for (ida, len, was, now) in changed.iter().take(LOGGED) {
        crate::framework::log(&format!(
            "  {ida:#x} +{len:#x}: was {}; now {}",
            hex(was),
            hex(now)
        ));
    }
}
