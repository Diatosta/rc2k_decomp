//! The hook and self-test machinery for functions that only change game variables. Shadow
//! mode cannot simply run such a function twice, so it runs the Rust version, notes what it
//! left, puts the variables back, runs the original and compares.

/// A piece of game memory a function may change: (address, bytes).
pub type Region = (u32, usize);

pub fn capture(regions: &[Region]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for &(addr, len) in regions {
        bytes.extend_from_slice(
            &crate::pointers::GamePtr::new(addr)
                .read_bytes(len, "a captured region")
                .unwrap_or_else(|e| crate::fatal::raise::end("service::harness", e)),
        );
    }
    bytes
}

pub fn restore(regions: &[Region], mut bytes: &[u8]) {
    for &(addr, len) in regions {
        crate::pointers::GamePtr::new(addr)
            .write_bytes(&bytes[..len], "a captured region")
            .unwrap_or_else(|e| crate::fatal::raise::end("service::harness", e));
        bytes = &bytes[len..];
    }
}
