//! The ground's faces (converted 2026-10-21 from `physics::face_has_edge` 0x45DA72).

/// `physics::face_has_edge`: whether the edge (`a`, `b`) is one of a face's, by its
/// corners p1 .. p4 (the face's +4 .. +0x10): (p1, p3), (p2, p1), (p4, p2) or (p3, p4).
pub fn has_edge(a: u32, b: u32, p: [u32; 4]) -> bool {
    [(0, 2), (1, 0), (3, 1), (2, 3)]
        .iter()
        .any(|&(x, y)| a == p[x] && b == p[y])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges() {
        let p = [10, 20, 30, 40];
        assert!(has_edge(10, 30, p));
        assert!(has_edge(20, 10, p));
        assert!(has_edge(40, 20, p));
        assert!(has_edge(30, 40, p));
        assert!(!has_edge(30, 10, p));
        assert!(!has_edge(10, 20, p));
    }
}
