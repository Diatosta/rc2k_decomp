//! One-off tools of the move off addresses, run while `data/ral_image.bin` and
//! `moved::MOVED` exist (both go with this file):
//!
//! - `RC2K_FREEZE_FIRST_VALUES=<golden> RC2K_FIRST_VALUES_MANIFEST=<manifest>`: writes the
//!   golden file: every registry store's size and hash from the image, and each manifest
//!   line (`<key> <IDA address> <length>`, hex) as a named read with its bytes.
//! - `RC2K_NAME_ADDRESSES=<in> RC2K_ADDRESS_NAMES=<out>`: each IDA address of `<in>` (one
//!   hex number first on each line) named by store key, offset and field path, or `image`
//!   where no store holds it (`tools/m7_tests/codemod.py` reads the result).

use super::first_values::{hash, store_keys};

/// A named read longer than this many bytes is frozen as size and hash only.
const LONGEST_HELD: usize = 0x4000;

fn hex(n: &str) -> usize {
    usize::from_str_radix(n.trim_start_matches("0x").replace('_', "").as_str(), 16)
        .unwrap_or_else(|_| panic!("a hex number: {n}"))
}

/// The golden key of each registry store, by the host address of its first byte.
fn keys_by_first() -> std::collections::HashMap<usize, String> {
    store_keys()
        .into_iter()
        .map(|(k, e)| (e.store.first().addr(), k))
        .collect()
}

#[test]
fn freeze_first_values() {
    let (Ok(out), Ok(manifest)) = (
        std::env::var("RC2K_FREEZE_FIRST_VALUES"),
        std::env::var("RC2K_FIRST_VALUES_MANIFEST"),
    ) else {
        return;
    };
    let keys = keys_by_first();
    let mut lines = Vec::new();
    for m in crate::moved::MOVED.iter() {
        let first = m.store().first().addr();
        let key = keys
            .get(&first)
            .expect("every moved range is a registry store");
        let image = crate::moved::image_bytes(m.ida, m.len);
        lines.push(format!("{key} {} {:016x}", m.len, hash(&image)));
    }
    let manifest = std::fs::read_to_string(manifest).expect("the manifest");
    for line in manifest.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let w: Vec<&str> = line.split_whitespace().collect();
        let [key, ida, len] = w[..] else {
            panic!("a manifest line is `key ida len`: {line}");
        };
        let image = crate::moved::image_bytes(hex(ida), hex(len));
        // A long read is held as size and hash only (`first_values::holds`).
        let bytes: String = if image.len() > LONGEST_HELD {
            String::new()
        } else {
            image
                .iter()
                .map(|b| format!(" {b:02x}"))
                .collect::<String>()
                .replace(' ', "")
        };
        let line = format!("{key} {} {:016x}", image.len(), hash(&image));
        lines.push(if bytes.is_empty() {
            line
        } else {
            format!("{line} {bytes}")
        });
    }
    lines.sort();
    let n = lines.len();
    lines.dedup();
    assert_eq!(n, lines.len(), "a key twice");
    let mut text = String::from(
        "# The game's first values (test_support::first_values). Made by\n\
         # tools/m7_tests/freeze.sh; a layout change updates it in the same commit.\n",
    );
    for l in lines {
        text.push_str(&l);
        text.push('\n');
    }
    std::fs::write(out, text).expect("the golden file");
}

#[test]
fn name_addresses() {
    let (Ok(input), Ok(out)) = (
        std::env::var("RC2K_NAME_ADDRESSES"),
        std::env::var("RC2K_ADDRESS_NAMES"),
    ) else {
        return;
    };
    let keys = keys_by_first();
    let mut text = String::new();
    for line in std::fs::read_to_string(input)
        .expect("the addresses")
        .lines()
    {
        let Some(word) = line.split_whitespace().next() else {
            continue;
        };
        let ida = hex(word);
        let named = ida
            .checked_sub(crate::framework::IMAGE_BASE)
            .and_then(crate::moved::home_of_rva)
            .and_then(|home| crate::stores::find(crate::pointers::host_at(home).addr()));
        match named {
            Some(f) => {
                let start = f.store.first().addr();
                let key = keys.get(&start).cloned().unwrap_or_default();
                text.push_str(&format!(
                    "{ida:#x} {key} {:#x} {}\n",
                    f.offset,
                    if f.field().is_empty() {
                        "-".into()
                    } else {
                        f.field()
                    }
                ));
            }
            None => text.push_str(&format!("{ida:#x} image\n")),
        }
    }
    std::fs::write(out, text).expect("the names");
}
