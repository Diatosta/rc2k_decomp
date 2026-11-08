//! The order an object's listed faces are drawn in (converted 2026-10-21 from
//! `scene::sort_listed_faces` 0x45098C and its quicksort 0x4509BF): the largest key
//! first. The quicksort is the original's step for step, since which of two equal keys
//! comes first is its doing.

use crate::Invalid;

/// A listed face: 0x10 bytes, the key (compared unsigned) the second dword.
pub type ListedFace = [u32; 4];

/// The most faces the list holds.
pub const MOST: usize = 0x1000;

fn key(list: &[ListedFace], at: usize) -> Result<u32, Invalid> {
    list.get(at)
        .map(|f| f[1])
        .ok_or(Invalid("the faces' sort went past the list"))
}

/// 0x4509BF on the entries `low` ..= `high`: the key in the middle is the pivot; the
/// left place moves on one entry if its key is above the pivot, the right place moves
/// back while its key is below; if the two places are not the same the entries change
/// places, and the left one moves on once more if their keys are now equal; when they
/// meet, the part before and the part after are sorted the same way.
fn sort(list: &mut [ListedFace], low: usize, high: usize) -> Result<(), Invalid> {
    if low >= high {
        return Ok(());
    }
    let (mut left, mut right) = (low, high);
    let pivot = key(list, low + (high - low) / 2)?;
    loop {
        if pivot < key(list, left)? {
            left += 1;
        }
        while pivot > key(list, right)? {
            right = right
                .checked_sub(1)
                .ok_or(Invalid("the faces' sort went past the list"))?;
        }
        if left == right {
            break;
        }
        key(list, left)?;
        list.swap(left, right);
        if list[left][1] == list[right][1] {
            left += 1;
        }
    }
    if let Some(before) = left.checked_sub(1)
        && low < before
    {
        sort(list, low, before)?;
    }
    if left + 1 < high {
        sort(list, left + 1, high)?;
    }
    Ok(())
}

/// `sort_listed_faces`: the list sorted, the largest key first (nothing with fewer than
/// two). Refuses a list longer than `MOST`, and a sort that would go past the list (the
/// original would read and swap outside it), both before anything is changed.
pub fn sort_listed_faces(list: &mut [ListedFace]) -> Result<(), Invalid> {
    if list.len() > MOST {
        return Err(Invalid("more faces listed than the list holds"));
    }
    if list.len() < 2 {
        return Ok(());
    }
    let mut sorted = list.to_vec();
    let high = sorted.len() - 1;
    sort(&mut sorted, 0, high)?;
    list.copy_from_slice(&sorted);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts() {
        let mut seed = 0x1234_5678u32;
        for len in 0..200usize {
            for spread in [2u32, 5, 1000] {
                let mut list: Vec<ListedFace> = (0..len)
                    .map(|n| {
                        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                        [n as u32, (seed >> 8) % spread, 0, 0]
                    })
                    .collect();
                let mut keys: Vec<u32> = list.iter().map(|f| f[1]).collect();
                sort_listed_faces(&mut list).unwrap();
                keys.sort_unstable_by(|a, b| b.cmp(a));
                assert_eq!(list.iter().map(|f| f[1]).collect::<Vec<_>>(), keys);
            }
        }
        let mut list = [[0, 9, 0, 0], [1, 5, 0, 0], [2, 7, 0, 0]];
        sort_listed_faces(&mut list).unwrap();
        assert_eq!(list.map(|f| f[0]), [0, 2, 1]);
        assert!(sort_listed_faces(&mut vec![[0; 4]; MOST + 1]).is_err());
    }
}
