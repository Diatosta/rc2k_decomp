//! Quicksort of fixed-size records by one dword (unsigned). The steps are kept exactly
//! because the order of equal keys depends on them: the middle key is the pivot; the left
//! place advances while its key belongs before the pivot, the right place retreats while its
//! key belongs after it; unless they meet the records swap and the left place advances once
//! more if the keys are now equal; then both parts are sorted the same way. The recursion
//! (as deep as the list on equal keys) is a list of returns.

use crate::Invalid;

/// The order the records are sorted in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// The smallest key first.
    Up,
    /// The largest key first.
    Down,
}

impl Order {
    /// Whether a key belongs before the pivot (the left place moves on past it).
    fn before(self, pivot: u32, key: u32) -> bool {
        match self {
            Order::Up => pivot > key,
            Order::Down => pivot < key,
        }
    }

    /// Whether a key belongs after the pivot (the right place moves back past it).
    fn after(self, pivot: u32, key: u32) -> bool {
        match self {
            Order::Up => pivot < key,
            Order::Down => pivot > key,
        }
    }
}

/// What the last partition left: the pivot, the last key read and the two places.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scan {
    /// The pivot.
    pub pivot: u32,
    /// The last key read.
    pub key: u32,
    /// The left place.
    pub left: i64,
    /// The right place.
    pub right: i64,
}

/// Where the sort ends: the last part's first and last places.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ends {
    /// The last part's first place.
    pub first: i64,
    /// The last part's last place.
    pub last: i64,
    pub scan: Option<Scan>,
    /// The last record's last dword before it was swapped (none without a swap).
    pub swapped: Option<u32>,
}

const OUTSIDE: Invalid = Invalid("the sort went past the list");

fn key_at<const N: usize>(list: &[[u32; N]], at: i64, key: usize) -> Result<u32, Invalid> {
    usize::try_from(at)
        .ok()
        .and_then(|a| list.get(a))
        .map(|r| r[key])
        .ok_or(OUTSIDE)
}

/// Sorts the list by dword `key` of its records in `order`. Refuses a key outside the record
/// and a sort that would go past the list, both before anything changes.
pub fn quick_sort<const N: usize>(
    list: &mut [[u32; N]],
    key: usize,
    order: Order,
) -> Result<Ends, Invalid> {
    if key >= N {
        return Err(Invalid("the sort's key is outside the record"));
    }
    enum Back {
        First(i64),
        Second,
    }
    enum Step {
        Enter,
        AfterFirst(i64),
        Return,
    }
    let mut sorted = list.to_vec();
    let mut ends = Ends {
        first: 0,
        last: list.len() as i64 - 1,
        scan: None,
        swapped: None,
    };
    let mut calls: Vec<Back> = Vec::new();
    let mut step = Step::Enter;
    loop {
        step = match step {
            Step::Enter => {
                if ends.first >= ends.last {
                    Step::Return
                } else {
                    let (mut left, mut right) = (ends.first, ends.last);
                    let pivot = key_at(&sorted, left + (right - left) / 2, key)?;
                    let mut last_key;
                    loop {
                        last_key = key_at(&sorted, left, key)?;
                        if order.before(pivot, last_key) {
                            left += 1;
                        }
                        loop {
                            last_key = key_at(&sorted, right, key)?;
                            right -= 1;
                            if !order.after(pivot, last_key) {
                                break;
                            }
                        }
                        right += 1;
                        if left == right {
                            break;
                        }
                        let (l, r) = (left as usize, right as usize);
                        key_at(&sorted, left, key)?;
                        ends.swapped = Some(sorted[l][N - 1]);
                        sorted.swap(l, r);
                        last_key = sorted[l][key];
                        if last_key == sorted[r][key] {
                            left += 1;
                        }
                    }
                    ends.scan = Some(Scan {
                        pivot,
                        key: last_key,
                        left,
                        right,
                    });
                    let outer = ends.last;
                    ends.last = left - 1;
                    if ends.first < ends.last {
                        calls.push(Back::First(outer));
                        Step::Enter
                    } else {
                        Step::AfterFirst(outer)
                    }
                }
            }
            Step::AfterFirst(outer) => {
                ends.first = ends.last + 2;
                ends.last = outer;
                if ends.first < ends.last {
                    calls.push(Back::Second);
                    Step::Enter
                } else {
                    Step::Return
                }
            }
            Step::Return => match calls.pop() {
                None => break,
                Some(Back::First(outer)) => Step::AfterFirst(outer),
                Some(Back::Second) => Step::Return,
            },
        };
    }
    list.copy_from_slice(&sorted);
    Ok(ends)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lot(seed: &mut u32) -> u32 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 17;
        *seed ^= *seed << 5;
        *seed
    }

    #[test]
    fn sorts_both_ways() {
        let mut seed = 0x1234_5678u32;
        for len in 0..120usize {
            for spread in [1u32, 3, 50, u32::MAX] {
                for order in [Order::Up, Order::Down] {
                    let mut list: Vec<[u32; 3]> = (0..len)
                        .map(|n| [n as u32, lot(&mut seed) % spread.max(1), 7])
                        .collect();
                    let mut keys: Vec<u32> = list.iter().map(|r| r[1]).collect();
                    quick_sort(&mut list, 1, order).unwrap();
                    keys.sort_unstable();
                    if order == Order::Down {
                        keys.reverse();
                    }
                    assert_eq!(list.iter().map(|r| r[1]).collect::<Vec<_>>(), keys);
                    let mut ids: Vec<u32> = list.iter().map(|r| r[0]).collect();
                    ids.sort_unstable();
                    assert_eq!(ids, (0..len as u32).collect::<Vec<_>>());
                }
            }
        }
    }

    #[test]
    fn ends_left() {
        // Two records in order: one partition, no swap, empty right part.
        let mut list = [[1u32, 10], [2, 20]];
        let ends = quick_sort(&mut list, 1, Order::Up).unwrap();
        assert_eq!(list, [[1, 10], [2, 20]]);
        assert_eq!(ends.swapped, None);
        let scan = ends.scan.unwrap();
        assert_eq!(scan.pivot, 10);
        // Out of order: one swap.
        let mut list = [[1u32, 20], [2, 10]];
        let ends = quick_sort(&mut list, 1, Order::Up).unwrap();
        assert_eq!(list, [[2, 10], [1, 20]]);
        assert_eq!(ends.swapped, Some(20));
        // Nothing to sort.
        let mut one = [[5u32, 5]];
        let ends = quick_sort(&mut one, 0, Order::Down).unwrap();
        assert_eq!((ends.first, ends.last, ends.scan), (0, 0, None));
        assert!(quick_sort(&mut one, 2, Order::Down).is_err());
    }

    fn reference(mem: &mut [u32], n: u32, key: u32, down: bool, r: &mut [u32; 7]) {
        let size = 4 * n;
        let get = |m: &[u32], a: u32| m[((a - 0x1000) / 4) as usize];
        let [pivot, value, low, high, first, last, swap] = r;
        if *first >= *last {
            return;
        }
        *low = *first;
        *high = *last;
        *pivot = get(mem, (*high - *low) / 2 / size * size + *low + 4 * key);
        loop {
            *value = get(mem, *low + 4 * key);
            *low += size;
            let stays = if down {
                *pivot < *value
            } else {
                *pivot > *value
            };
            if !stays {
                *low -= size;
            }
            loop {
                *value = get(mem, *high + 4 * key);
                *high = high.wrapping_sub(size);
                let on = if down {
                    *pivot > *value
                } else {
                    *pivot < *value
                };
                if !on {
                    break;
                }
            }
            *high = high.wrapping_add(size);
            if *low == *high {
                break;
            }
            for k in 0..n {
                *swap = get(mem, *low + 4 * k);
                *value = get(mem, *high + 4 * k);
                mem[((*high + 4 * k - 0x1000) / 4) as usize] = *swap;
                mem[((*low + 4 * k - 0x1000) / 4) as usize] = *value;
            }
            *value = get(mem, *low + 4 * key);
            if *value == get(mem, *high + 4 * key) {
                *low += size;
            }
        }
        let outer = *last;
        *last = low.wrapping_sub(size);
        let mut regs = [*pivot, *value, *low, *high, *first, *last, *swap];
        if regs[4] < regs[5] {
            reference(mem, n, key, down, &mut regs);
        }
        regs[4] = regs[5].wrapping_add(2 * size);
        regs[5] = outer;
        if regs[4] < regs[5] {
            reference(mem, n, key, down, &mut regs);
        }
        *r = regs;
    }

    #[test]
    fn the_originals_steps() {
        let mut seed = 0x0BAD_F00Du32;
        for len in 2..60usize {
            for spread in [1u32, 2, 7, u32::MAX] {
                for order in [Order::Up, Order::Down] {
                    let mut list: Vec<[u32; 3]> = (0..len)
                        .map(|_| {
                            let k = lot(&mut seed) % spread.max(1);
                            [lot(&mut seed), k, lot(&mut seed)]
                        })
                        .collect();
                    let mut mem: Vec<u32> = list.iter().flatten().copied().collect();
                    let mut r = [0, 0, 0, 0, 0x1000, 0x1000 + 12 * (len as u32 - 1), 0xEBEB];
                    reference(&mut mem, 3, 1, order == Order::Down, &mut r);
                    let ends = quick_sort(&mut list, 1, order).unwrap();
                    assert_eq!(list.iter().flatten().copied().collect::<Vec<_>>(), mem);
                    let place = |n: i64| (0x1000 + 12 * n) as u32;
                    let scan = ends.scan.unwrap();
                    assert_eq!(
                        [
                            scan.pivot,
                            scan.key,
                            place(scan.left),
                            place(scan.right),
                            place(ends.first),
                            place(ends.last),
                            ends.swapped.unwrap_or(0xEBEB)
                        ],
                        r,
                        "{len} {spread} {order:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn equal_keys_deep() {
        // Miri: this case is quadratic, and took hours at the full length
        let mut list = vec![[3u32, 0]; if cfg!(miri) { 0x40 } else { 0x1000 }];
        for (n, r) in list.iter_mut().enumerate() {
            r[1] = n as u32;
        }
        quick_sort(&mut list, 0, Order::Down).unwrap();
        assert!(list.iter().all(|r| r[0] == 3));
    }
}
