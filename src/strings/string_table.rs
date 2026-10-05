use crate::framework::game_addr;
use std::ffi::{CStr, c_char};
use std::ops::RangeInclusive;

const BLOB_PTR_RVA: usize = 0x4E07D4 - 0x400000; // dword_4E07D4: pointer to the packed blob
const PTRS_HIGH_RVA: usize = 0x525DE8 - 0x400000; // dword_525DE8[id], ids with bit 0x1000
const PTRS_LOW_RVA: usize = 0x95D664 - 0x400000; // src[id], ids 0x5A5..=0x658

pub const LOW_IDS: RangeInclusive<u32> = 0x5A5..=0x658;

/// The game fills this pointer in at runtime, so read it on every use.
pub fn blob() -> *const u8 {
    unsafe { (game_addr(BLOB_PTR_RVA) as *const u32).read_volatile() as usize as *const u8 }
}

unsafe fn table_entry(rva: usize, id: u32) -> *const u8 {
    unsafe {
        (game_addr(rva) as *const u32)
            .wrapping_add(id as usize)
            .read_volatile() as usize as *const u8
    }
}
pub unsafe fn ptr_high(id: u32) -> *const u8 {
    unsafe { table_entry(PTRS_HIGH_RVA, id) }
}
pub unsafe fn ptr_low(id: u32) -> *const u8 {
    unsafe { table_entry(PTRS_LOW_RVA, id) }
}

/// Entry = 4 bytes at blob + 4*id: [flag: 8 bits | offset: 24 bits].
/// flag < 0x80: `flag` literal bytes at blob + offset.
/// flag >= 0x80: (256 - flag) 16-bit child ids at blob + offset, expanded in order.
/// No cycle or bounds check, like the original.
pub unsafe fn expand(blob: *const u8, id: u32, emit: &mut impl FnMut(*const u8, usize)) {
    unsafe {
        let entry = blob
            .wrapping_add((id as usize).wrapping_mul(4))
            .cast::<u32>()
            .read_unaligned();
        let flag = (entry >> 24) as u8;
        let data = blob.wrapping_add((entry & 0x00FF_FFFF) as usize);
        if flag < 0x80 {
            emit(data, flag as usize);
        } else {
            for i in 0..(256 - flag as usize) {
                let child = data.wrapping_add(2 * i).cast::<u16>().read_unaligned();
                expand(blob, child as u32, emit);
            }
        }
    }
}

/// Rust version of append_string_by_id. Returns the `eax` the original leaves behind:
/// the string pointer for the two pointer-table branches, the id itself for the fallback.
pub unsafe fn append_by_id(id: u32, emit: &mut impl FnMut(*const u8, usize)) -> u32 {
    unsafe {
        let direct = if id & 0x1000 != 0 {
            Some(ptr_high(id))
        } else if LOW_IDS.contains(&id) {
            let p = ptr_low(id);
            (!p.is_null()).then_some(p)
        } else {
            None
        };
        match direct {
            Some(p) => {
                let len = CStr::from_ptr(p as *const c_char).to_bytes().len();
                emit(p, len);
                p as usize as u32
            }
            None => {
                expand(blob(), id, emit);
                id
            }
        }
    }
}

/// Assumes the 4-byte entry index sits at the start of the blob, so entry 0's offset
/// (where the data begins) / 4 is the entry count. Verify this in IDA.
pub fn entry_count() -> usize {
    let b = blob();
    if b.is_null() {
        return 0;
    }
    let first = unsafe { b.cast::<u32>().read_unaligned() } & 0x00FF_FFFF;
    ((first / 4) as usize).min(0x10000)
}
