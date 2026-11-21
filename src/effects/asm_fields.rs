//! The marks' stores as the bytes of their fields in order, for the differential tests of the
//! kept x87 assembly (`asm_layout`, `typed_sync`). The tests own this layer: each store lists its
//! fields once (`plain_fields!`) and is then read out as, and filled from, a flat byte image, one
//! little-endian word per cell. Nothing here walks a store by path or needs the store to be a
//! record of cells beyond the few leaf types implemented below. Test code only.

use crate::memory::PlainList;
use crate::memory::plain_buffer::PlainBuffer;
use crate::memory::session::{
    Cell32, PlainByte, PlainDword, PlainFlag, PlainFloat, PlainInt, PlainWord,
};

/// A value held as bytes: a cell, an array of them, or a record listed with `plain_fields!`.
pub trait Leaf {
    /// Its size in bytes.
    fn byte_len(&self) -> usize;
    /// Appends its bytes to `out`.
    fn dump(&self, out: &mut Vec<u8>);
    /// Takes its bytes from the front of `from`, which then starts after them.
    fn load(&self, from: &mut &[u8]);
}

/// A store listed field by field (`plain_fields!`).
pub trait Fields: Leaf + Sized {
    /// Each field's name (`.name`) and offset in the image, in order.
    fn starts(&self) -> Vec<(&'static str, usize)>;
    /// A store of zeros, on the heap (some stores are near 100 KB, and a test thread's stack is
    /// small).
    fn zeroed() -> Box<Self>;
}

/// Takes `N` bytes off the front of `from`.
fn take<const N: usize>(from: &mut &[u8]) -> [u8; N] {
    let (head, rest) = from.split_at(N);
    *from = rest;
    head.try_into().expect("N bytes")
}

macro_rules! leaf_of_dword {
    ($($t:ty),*) => {$(
        impl Leaf for $t {
            fn byte_len(&self) -> usize {
                4
            }
            fn dump(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.bits().to_le_bytes());
            }
            fn load(&self, from: &mut &[u8]) {
                self.set_bits(u32::from_le_bytes(take::<4>(from)));
            }
        }
    )*};
}
leaf_of_dword!(PlainDword, PlainFloat, PlainInt);

impl Leaf for PlainFlag {
    fn byte_len(&self) -> usize {
        4
    }
    fn dump(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.cell().get().to_le_bytes());
    }
    fn load(&self, from: &mut &[u8]) {
        self.cell().set(u32::from_le_bytes(take::<4>(from)));
    }
}

impl Leaf for PlainWord {
    fn byte_len(&self) -> usize {
        2
    }
    fn dump(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.get().to_le_bytes());
    }
    fn load(&self, from: &mut &[u8]) {
        self.set(u16::from_le_bytes(take::<2>(from)));
    }
}

impl Leaf for PlainByte {
    fn byte_len(&self) -> usize {
        1
    }
    fn dump(&self, out: &mut Vec<u8>) {
        out.push(self.get());
    }
    fn load(&self, from: &mut &[u8]) {
        self.set(take::<1>(from)[0]);
    }
}

impl<const N: usize> Leaf for PlainBuffer<N> {
    fn byte_len(&self) -> usize {
        N
    }
    fn dump(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&PlainBuffer::read(self));
    }
    fn load(&self, from: &mut &[u8]) {
        let (head, rest) = from.split_at(N);
        *from = rest;
        PlainBuffer::write(self, head.try_into().expect("N bytes"));
    }
}

/// A memory list: its four dwords in order.
impl Leaf for PlainList {
    fn byte_len(&self) -> usize {
        16
    }
    fn dump(&self, out: &mut Vec<u8>) {
        let list = self.get();
        for word in [list.head, list.total, list.cursor, list.remaining] {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
    fn load(&self, from: &mut &[u8]) {
        let mut list = self.get();
        for word in [
            &mut list.head,
            &mut list.total,
            &mut list.cursor,
            &mut list.remaining,
        ] {
            *word = u32::from_le_bytes(take::<4>(from));
        }
        self.set(list);
    }
}

impl<T: Leaf, const N: usize> Leaf for [T; N] {
    fn byte_len(&self) -> usize {
        self.iter().map(Leaf::byte_len).sum()
    }
    fn dump(&self, out: &mut Vec<u8>) {
        self.iter().for_each(|x| x.dump(out));
    }
    fn load(&self, from: &mut &[u8]) {
        self.iter().for_each(|x| x.load(from));
    }
}

/// A plain value held as bytes: a number, an array of them, or a plain record listed with
/// `plain_fields!`. Unlike `Leaf` it is read and filled through `&mut`.
pub trait PlainLeaf {
    /// Its size in bytes.
    fn byte_len(&self) -> usize;
    /// Appends its bytes to `out`.
    fn dump(&self, out: &mut Vec<u8>);
    /// Takes its bytes from the front of `from`, which then starts after them.
    fn load(&mut self, from: &mut &[u8]);
}

macro_rules! plain_leaf_of_number {
    ($($t:ty),*) => {$(
        impl PlainLeaf for $t {
            fn byte_len(&self) -> usize {
                std::mem::size_of::<$t>()
            }
            fn dump(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_le_bytes());
            }
            fn load(&mut self, from: &mut &[u8]) {
                *self = <$t>::from_le_bytes(take::<{ std::mem::size_of::<$t>() }>(from));
            }
        }
    )*};
}
plain_leaf_of_number!(u8, u16, u32, i32, f32);

impl<T: PlainLeaf, const N: usize> PlainLeaf for [T; N] {
    fn byte_len(&self) -> usize {
        self.iter().map(PlainLeaf::byte_len).sum()
    }
    fn dump(&self, out: &mut Vec<u8>) {
        self.iter().for_each(|x| x.dump(out));
    }
    fn load(&mut self, from: &mut &[u8]) {
        self.iter_mut().for_each(|x| x.load(from));
    }
}

/// Lists the fields of a plain store held in a `RefCell`, in order: the cell is then a `Leaf` and
/// `Fields` (its fields' bytes one after another). The store needs a `const fn new()`.
macro_rules! plain_fields {
    ($t:ident { $($f:ident),* $(,)? }) => {
        impl crate::effects::asm_fields::Leaf for std::cell::RefCell<$t> {
            fn byte_len(&self) -> usize {
                let store = self.borrow();
                0 $(+ crate::effects::asm_fields::PlainLeaf::byte_len(&store.$f))*
            }
            fn dump(&self, out: &mut Vec<u8>) {
                let store = self.borrow();
                $(crate::effects::asm_fields::PlainLeaf::dump(&store.$f, out);)*
            }
            fn load(&self, from: &mut &[u8]) {
                let mut store = self.borrow_mut();
                $(crate::effects::asm_fields::PlainLeaf::load(&mut store.$f, from);)*
            }
        }
        impl crate::effects::asm_fields::Fields for std::cell::RefCell<$t> {
            fn starts(&self) -> Vec<(&'static str, usize)> {
                let store = self.borrow();
                let mut at = 0;
                let mut starts = Vec::new();
                $(
                    starts.push((concat!(".", stringify!($f)), at));
                    at += crate::effects::asm_fields::PlainLeaf::byte_len(&store.$f);
                )*
                let _ = at;
                starts
            }
            fn zeroed() -> Box<Self> {
                let store = Box::new(std::cell::RefCell::new(<$t>::new()));
                let zeros = vec![0u8; crate::effects::asm_fields::Leaf::byte_len(&*store)];
                crate::effects::asm_fields::put_image(&*store, &zeros);
                store
            }
        }
    };
}
pub(crate) use plain_fields;

/// All of `store`'s bytes.
pub fn image<T: Leaf>(store: &T) -> Vec<u8> {
    let mut out = Vec::with_capacity(store.byte_len());
    store.dump(&mut out);
    out
}

/// `store`'s fields filled from the front of `bytes`, which must be exactly its size.
pub fn put_image<T: Leaf>(store: &T, bytes: &[u8]) {
    assert_eq!(bytes.len(), store.byte_len(), "the store's size");
    let mut from = bytes;
    store.load(&mut from);
}

/// Where the field `name` begins in the store's image.
pub fn field_start<T: Fields>(store: &T, name: &str) -> Option<usize> {
    store
        .starts()
        .into_iter()
        .find_map(|(n, at)| (n == name).then_some(at))
}
