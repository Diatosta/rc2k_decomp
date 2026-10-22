//! The game's memory lists and the figures it keeps about them (converted 2026-10-21 from
//! `memory::allocate` 0x4028A7, `memory::free_list` 0x4028F5, `memory::take_from_list`
//! 0x402956 and `memory::rewind_list` 0x402941).
//!
//! Every user of the allocator has a list of its own: the blocks taken on it are chained
//! through a header in front of each (+0 the block taken before on the same list, +4 the
//! bytes asked for), and are all given back in one go. Some users hand out the newest
//! block's memory piece by piece (`take`, after `rewind`).

/// The header in front of each block's memory.
pub const HEADER_LEN: u32 = 0x40;

/// A list of blocks, as the game lays it out (16 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct List {
    /// The block taken last (its header), or 0.
    pub head: u32,
    /// The bytes asked for, over all blocks.
    pub total: u32,
    /// Set by `rewind`: where the next piece of the newest block starts, and the bytes
    /// left to hand out. `emptied` leaves both 0.
    pub cursor: u32,
    pub remaining: u32,
}

/// The bytes a block of `bytes` takes with its header, or None when that wraps to 0 (the
/// one size the original cannot take).
pub fn block_size(bytes: u32) -> Option<u32> {
    match bytes.wrapping_add(HEADER_LEN) {
        0 => None,
        whole => Some(whole),
    }
}

impl List {
    /// `memory::allocate`: the block at `block` (its header) put on the list for `bytes`;
    /// the header's two dwords to write there.
    pub fn add(&mut self, block: u32, bytes: u32) -> [u32; 2] {
        let header = [self.head, bytes];
        self.head = block;
        self.total = self.total.wrapping_add(bytes);
        header
    }

    /// `memory::free_list`: the list emptied (all four fields 0); the newest block, whose
    /// header leads to the others.
    pub fn emptied(&mut self) -> u32 {
        let head = self.head;
        *self = List::default();
        head
    }

    /// `memory::take_from_list`: the address of the next `bytes` of the newest block, or
    /// None (nothing changed) if fewer are left.
    pub fn take(&mut self, bytes: u32) -> Option<u32> {
        if bytes > self.remaining {
            return None;
        }
        self.remaining -= bytes;
        let taken = self.cursor;
        self.cursor = taken.wrapping_add(bytes);
        Some(taken)
    }

    /// `memory::rewind_list`: the cursor back to the newest block's memory, and all of the
    /// list's bytes left.
    pub fn rewind(&mut self) {
        self.cursor = match self.head {
            0 => 0,
            head => head.wrapping_add(HEADER_LEN),
        };
        self.remaining = self.total;
    }
}

/// The three figures of the game's log ("MEM: memory turnover= .. maximum load= .."):
/// the most bytes held at once, the bytes held now, all bytes ever asked for (wrapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Figures {
    pub most_held: u32,
    pub held: u32,
    pub turnover: u32,
}

impl Figures {
    /// `memory::allocate`: `bytes` more taken (the most held compared unsigned).
    pub fn taken(&mut self, bytes: u32) {
        self.turnover = self.turnover.wrapping_add(bytes);
        self.held = self.held.wrapping_add(bytes);
        if self.held > self.most_held {
            self.most_held = self.held;
        }
    }

    /// `memory::free_list`: a list of `total` bytes given back (also an empty one's).
    pub fn given_back(&mut self, total: u32) {
        self.held = self.held.wrapping_sub(total);
    }
}

/// The game's own heap (the setting `altmemenable`; converted 2026-10-21 from
/// `memory::open_own_heap` 0x40298A and `memory::free_own_heap` 0x4029DB): in use only
/// when the switch is 1.
pub fn own_heap_on(switch: u32) -> bool {
    switch == 1
}

/// The smallest own heap, in Kb (signed).
pub const SMALLEST_HEAP_KB: i32 = 0x2800;

/// The own heap's size from the setting (`altmemsize`, Kb): the Kb, at least
/// `SMALLEST_HEAP_KB`, and the bytes (an `imul`, wrapping); None when the bytes wrap to 0
/// (the original writes at address 0 then).
pub fn own_heap_size(kb_setting: u32) -> Option<(u32, u32)> {
    let kb = (kb_setting as i32).max(SMALLEST_HEAP_KB) as u32;
    match kb.wrapping_mul(0x400) {
        0 => None,
        bytes => Some((kb, bytes)),
    }
}

/// The own heap's first two dwords: 0 and its bytes.
pub fn own_heap_head(bytes: u32) -> [u32; 2] {
    [0, bytes]
}

/// `memory::move_bytes` (0x46573F) inside one buffer: `count` bytes from `from` to `to`,
/// forwards, a dword at a time (each read before it is written) and then the last bytes
/// one by one. With `to` a little past `from` the copy repeats what was there, as the
/// original's `rep movsd` / `rep movsb` does. Both ranges must lie in `buf`.
pub fn move_forward(buf: &mut [u8], from: usize, to: usize, count: usize) {
    let (mut f, mut t) = (from, to);
    for _ in 0..count / 4 {
        let dword: [u8; 4] = buf[f..f + 4].try_into().unwrap();
        buf[t..t + 4].copy_from_slice(&dword);
        f += 4;
        t += 4;
    }
    for _ in 0..count % 4 {
        buf[t] = buf[f];
        f += 1;
        t += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_on_a_list() {
        let mut list = List::default();
        assert_eq!(list.add(0x1000, 0x20), [0, 0x20]);
        assert_eq!(list.add(0x2000, 0x30), [0x1000, 0x30]);
        assert_eq!((list.head, list.total), (0x2000, 0x50));
        list.rewind();
        assert_eq!((list.cursor, list.remaining), (0x2040, 0x50));
        assert_eq!(list.take(0x10), Some(0x2040));
        assert_eq!(list.take(0x41), None);
        assert_eq!(list.take(0x40), Some(0x2050));
        assert_eq!((list.cursor, list.remaining), (0x2090, 0));
        assert_eq!(list.emptied(), 0x2000);
        assert_eq!(list, List::default());
        list.rewind();
        assert_eq!((list.cursor, list.remaining), (0, 0));
    }

    #[test]
    fn sizes_and_figures() {
        assert_eq!(block_size(0), Some(0x40));
        assert_eq!(block_size(0xFFFF_FFC0), None);
        assert_eq!(block_size(0xFFFF_FFC1), Some(1));
        let mut f = Figures {
            most_held: 0x100,
            held: 0xF0,
            turnover: 0xFFFF_FFFF,
        };
        f.taken(0x08);
        assert_eq!(
            f,
            Figures {
                most_held: 0x100,
                held: 0xF8,
                turnover: 7
            }
        );
        f.taken(0x10);
        assert_eq!(f.most_held, 0x108);
        f.given_back(0x200);
        assert_eq!(f.held, 0x108u32.wrapping_sub(0x200));
        f.taken(0);
        // Compared unsigned: a held count gone below 0 is "more".
        assert_eq!(f.most_held, f.held);
    }

    #[test]
    fn own_heap() {
        assert!(own_heap_on(1) && !own_heap_on(2));
        assert_eq!(own_heap_size(0), Some((0x2800, 0xA0_0000)));
        assert_eq!(own_heap_size(0x8000_0000), Some((0x2800, 0xA0_0000)));
        assert_eq!(own_heap_size(0x4000), Some((0x4000, 0x100_0000)));
        assert_eq!(own_heap_size(0x40_0000), None);
        assert_eq!(own_heap_head(5), [0, 5]);
    }

    #[test]
    fn moves_forward() {
        let start: Vec<u8> = (0..16).collect();
        let mut b = start.clone();
        move_forward(&mut b, 0, 8, 7);
        assert_eq!(&b[8..15], &start[0..7]);
        assert_eq!(b[15], 15);
        // one byte on: each dword read before it is written, the bytes one by one
        let mut b = start.clone();
        move_forward(&mut b, 0, 1, 6);
        assert_eq!(&b[..8], &[0, 0, 1, 2, 3, 3, 3, 7]);
        // backwards in the buffer: a plain copy
        let mut b = start.clone();
        move_forward(&mut b, 4, 0, 9);
        assert_eq!(&b[..9], &start[4..13]);
        let mut b = start.clone();
        move_forward(&mut b, 3, 3, 0);
        assert_eq!(b, start);
    }
}
