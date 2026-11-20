//! The service area's repair choices.
//!
//! Menu items 0 .. 19 each have a flag (`Service::repair_flags[0]`, 1 chosen) and a
//! mark; both tables end with -1 at entry 20. Items 0, 1, 0x10 and 0x11 have no repair time.
//! The time budget less the chosen items' time decides what fits: at least 900 is free.

use crate::Invalid;

/// The items with a flag.
pub const ITEMS: usize = 20;
/// The service time that is free.
pub(crate) const FREE_TIME: i32 = 900;
/// A flag table with its end mark.
pub type Table = [u32; ITEMS + 1];

/// The end mark's place (looked for from entry 1).
pub fn end_of(table: &Table) -> Option<usize> {
    (1..=ITEMS).find(|&n| table[n] == u32::MAX)
}

/// Sets every entry before the end mark to 0 (entry 0 without looking at it). Refused (nothing
/// changed): no end mark.
pub fn clear(table: &mut Table) -> Result<(), Invalid> {
    let end = end_of(table).ok_or(Invalid("a flag table that has lost its end mark"))?;
    table[..end].fill(0);
    Ok(())
}

/// The summed `time` of every chosen item (entry 0 is not checked for the end mark).
/// Refused: a chosen item without a time, no end mark.
pub fn chosen_time(flags: &Table, time: impl Fn(usize) -> Option<i32>) -> Result<i32, Invalid> {
    let mut sum = 0i32;
    let mut item = 0;
    loop {
        if flags[item] != 0 {
            let t = time(item).ok_or(Invalid(
                "a chosen item has no repair time: the original reads through null",
            ))?;
            sum = sum.wrapping_add(t);
        }
        item += 1;
        if item > ITEMS {
            return Err(Invalid("the repair flag table has lost its end mark"));
        }
        if flags[item] == u32::MAX {
            return Ok(sum);
        }
    }
}

/// What toggling an item answers: it fits (3), overruns the budget (1) or has no time left (2).
pub(crate) const FITS: u32 = 3;
pub(crate) const OVERRUNS: u32 = 1;
pub const NO_TIME: u32 = 2;

/// Whether flipping `item`'s flag fits: the total with the flip, and the status; the flag
/// ends as it was, but at no time left it is 0. Refused (nothing changed): what
/// `chosen_time` refuses.
pub(crate) fn try_toggle(
    flags: &mut Table,
    item: usize,
    budget: i32,
    time: impl Fn(usize) -> Option<i32>,
) -> Result<(i32, u32), Invalid> {
    let before = flags[item];
    let mut with = *flags;
    with[item] = before ^ 1;
    let total = chosen_time(&with, time)?;
    let left = budget.wrapping_sub(total);
    let (flag, status) = if left >= FREE_TIME {
        (before, FITS)
    } else if left <= 0 {
        (0, NO_TIME)
    } else {
        (before, OVERRUNS)
    };
    flags[item] = flag;
    Ok((total, status))
}

/// The chosen items' total; with free time left the marks cleared. Refused (nothing
/// changed): what `chosen_time` and `clear` refuse.
pub fn update_total(
    flags: &Table,
    marks: &mut Table,
    budget: i32,
    time: impl Fn(usize) -> Option<i32>,
) -> Result<i32, Invalid> {
    let total = chosen_time(flags, time)?;
    if budget.wrapping_sub(total) >= FREE_TIME {
        clear(marks)?;
    }
    Ok(total)
}

/// A repair item's action: its flag flipped and by what is left (budget less the total,
/// signed): free time, every mark 0; 0 .. 899, its mark 1; below 0, the flip undone and
/// the status 1 (else 0). The total, and the status. Refused (nothing changed): what
/// `chosen_time` refuses, a mark table without its end.
pub fn toggle_choice(
    flags: &mut Table,
    marks: &mut Table,
    item: usize,
    budget: i32,
    time: impl Fn(usize) -> Option<i32> + Copy,
) -> Result<(i32, u32), Invalid> {
    let (mut f, mut m) = (*flags, *marks);
    if end_of(&m).is_none() {
        return Err(Invalid("the repair mark table has lost its end mark"));
    }
    let mut status = 0;
    f[item] ^= 1;
    let total = chosen_time(&f, time)?;
    let left = budget.wrapping_sub(total);
    if left >= FREE_TIME {
        clear(&mut m)?;
    } else if left >= 0 {
        m[item] = 1;
    } else {
        f[item] ^= 1;
        status = OVERRUNS;
    }
    let total = chosen_time(&f, time)?;
    (*flags, *marks) = (f, m);
    Ok((total, status))
}

/// A setup value stepped: down stops at 0 (a negative one goes on down), up stops at `max`
/// (a value above it becomes `max`).
pub fn step(now: i32, max: i32, up: bool) -> i32 {
    if up {
        if now < max { now.wrapping_add(1) } else { max }
    } else if now != 0 {
        now.wrapping_sub(1)
    } else {
        0
    }
}

/// A change's selection flipped: true when the chosen value goes back to the installed one
/// (the flag then not 1).
pub fn toggle_change(flags: &mut Table, item: usize) -> bool {
    flags[item] ^= 1;
    flags[item] != 1
}

/// Whether an edit page opens: the item chosen already, or the budget still covers the
/// total and its time (signed, at least 0).
pub fn may_edit(flag: u32, total: i32, time: i32, budget: i32) -> bool {
    flag == 1 || budget.wrapping_sub(total.wrapping_add(time)) >= 0
}

/// A change page's value: the one chosen, the one installed, the one at the edit's start,
/// and (ride height and stiffness) the car's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Value {
    pub chosen: u32,
    pub installed: u32,
    pub entry: u32,
    pub in_car: Option<u32>,
}

/// What confirming a change page leaves: the chosen value, the status if trying the item
/// set it, and the total. The edit mode is always left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Confirmed {
    pub chosen: u32,
    pub status: Option<u32>,
    pub total: i32,
}

/// A change page confirmed: the value the car has already, the item's flag cleared; the
/// value unchanged in this edit, its selection flipped (`toggle_change`) and, unless that
/// selected it, that is all; else the flag cleared and `try_toggle` asked: no time left,
/// the chosen value back to the installed one; overrunning, chosen and marked; fitting,
/// the marks cleared and chosen. Then the total (`update_total`). Refused (nothing
/// changed): what those refuse.
pub fn confirm_change(
    flags: &mut Table,
    marks: &mut Table,
    item: usize,
    budget: i32,
    v: Value,
    time: impl Fn(usize) -> Option<i32> + Copy,
) -> Result<Confirmed, Invalid> {
    let (mut f, mut m) = (*flags, *marks);
    let mut chosen = v.chosen;
    let mut status = None;
    let tried = if v.in_car == Some(chosen) {
        f[item] = 0;
        false
    } else if v.entry == chosen {
        if toggle_change(&mut f, item) {
            chosen = v.installed;
        }
        f[item] == 1
    } else {
        true
    };
    if tried {
        f[item] = 0;
        let (_, st) = try_toggle(&mut f, item, budget, time)?;
        status = Some(st);
        match st {
            NO_TIME => chosen = v.installed,
            OVERRUNS => (f[item], m[item]) = (1, 1),
            _ => {
                clear(&mut m)?;
                f[item] = 1;
            }
        }
    }
    let total = update_total(&f, &mut m, budget, time)?;
    (*flags, *marks) = (f, m);
    Ok(Confirmed {
        chosen,
        status,
        total,
    })
}

/// `choose_repair` (`marked` false) or `choose_repair_marked`: the item's flag 1, and its
/// mark.
pub fn choose(flags: &mut Table, marks: &mut Table, item: usize, marked: bool) {
    flags[item] = 1;
    if marked {
        marks[item] = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Table {
        let mut t = [0; ITEMS + 1];
        t[ITEMS] = u32::MAX;
        t
    }

    #[test]
    fn repairs() {
        let time = |n: usize| (n >= 2).then_some(100 * n as i32);
        let mut flags = table();
        flags[2] = 1;
        flags[3] = 1;
        assert_eq!(chosen_time(&flags, time), Ok(500));
        flags[0] = 1;
        assert!(chosen_time(&flags, time).is_err());
        flags[0] = 0;
        // Budget 1500: flipping 4 makes 900 chosen, 600 left: overruns, the flag kept.
        assert_eq!(try_toggle(&mut flags, 4, 1500, time), Ok((900, OVERRUNS)));
        assert_eq!(flags[4], 0);
        assert_eq!(try_toggle(&mut flags, 4, 900, time), Ok((900, NO_TIME)));
        let mut marks = table();
        marks[5] = 1;
        assert_eq!(
            toggle_choice(&mut flags, &mut marks, 4, 1500, time),
            Ok((900, 0))
        );
        assert_eq!((flags[4], marks[4], marks[5]), (1, 1, 1));
        assert_eq!(
            toggle_choice(&mut flags, &mut marks, 9, 1500, time),
            Ok((900, OVERRUNS))
        );
        assert_eq!(flags[9], 0);
        assert_eq!(update_total(&flags, &mut marks, 5000, time), Ok(900));
        assert_eq!(marks, table());
        assert_eq!(
            (
                step(5, 5, true),
                step(9, 5, true),
                step(0, 5, false),
                step(-2, 5, false)
            ),
            (5, 5, 0, -3)
        );
        assert!(toggle_change(&mut flags, 4));
        assert!(
            may_edit(0, 600, 400, 1000) && !may_edit(0, 600, 401, 1000) && may_edit(1, 0, 0, -1)
        );
        let mut f = table();
        let mut m = table();
        let v = Value {
            chosen: 3,
            installed: 1,
            entry: 2,
            in_car: None,
        };
        // Budget 2000, item 4 (400): fits; chosen.
        assert_eq!(
            confirm_change(&mut f, &mut m, 4, 2000, v, time),
            Ok(Confirmed {
                chosen: 3,
                status: Some(FITS),
                total: 400
            })
        );
        assert_eq!(f[4], 1);
        // The car has it already: the flag cleared.
        let v2 = Value {
            in_car: Some(3),
            ..v
        };
        assert_eq!(
            confirm_change(&mut f, &mut m, 4, 2000, v2, time)
                .unwrap()
                .total,
            0
        );
        // No time: back to the installed value.
        assert_eq!(
            confirm_change(&mut f, &mut m, 4, 400, v, time)
                .unwrap()
                .chosen,
            1
        );
        let mut broken = [0; ITEMS + 1];
        assert!(clear(&mut broken).is_err());
    }
}
