//! The tests of the x87 arithmetic, one layer: `helpers` holds the plain versions production
//! no longer calls, `oracle` compares the typed helpers with the old ones, `recorded` checks
//! the results kept as hashes. `inputs` is their seeded random operands and comparison.

mod helpers;
mod inputs;
mod oracle;
mod recorded;

/// Every plain helper the tests call: production's `x87` and the test-only `helpers`.
mod plain {
    pub(crate) use super::helpers::*;
    pub(crate) use crate::math::x87::*;
}
