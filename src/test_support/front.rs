//! Helpers of the front end's tests (gfx, menu, scene, saves ...): a block whose first
//! values hold a handle where the game has an address.

use super::first_values::{assert_matches, golden, store_keys};

/// As `assert_block_starts_as_the_games`, with the block's bytes first put back to the
/// game's by `restore` (it writes the game's addresses over the places that hold handles).
pub fn assert_block_starts_as_the_games_but<T>(block: &'static T, restore: impl Fn(&mut [u8])) {
    let _g = crate::store::test_lock();
    let first = std::ptr::from_ref(block).addr();
    let (key, entry) = store_keys()
        .into_iter()
        .find(|(_, e)| e.store.first().addr() == first)
        .unwrap_or_else(|| panic!("no registry store starts at the block"));
    let mut mine = vec![0; golden(&key).size];
    entry.store.load(0, &mut mine).expect("a store's own bytes");
    restore(&mut mine);
    assert_matches(&key, &mine);
}
