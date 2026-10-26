//! The ground's faces (converted 2026-10-21 from `physics::face_has_edge` 0x45DA72 and
//! `physics::edge_side` 0x4425EF).

/// `physics::face_has_edge`: whether the edge (`a`, `b`) is one of a face's, by its
/// corners p1 .. p4 (the face's +4 .. +0x10): (p1, p3), (p2, p1), (p4, p2) or (p3, p4).
pub fn has_edge(a: u32, b: u32, p: [u32; 4]) -> bool {
    [(0, 2), (1, 0), (3, 1), (2, 3)]
        .iter()
        .any(|&(x, y)| a == p[x] && b == p[y])
}

/// `physics::edge_side`: which side of the edge from (x0, z0) to (x1, z1) the point (x, z)
/// lies (whole numbers): None when the ends are the same place; else (the second product
/// (z - z0)(x1 - x0), which the original keeps in `qword_620460`; whether
/// (x - x0)(z1 - z0) less it is negative). Each difference is a 32-bit `sub` taken as
/// signed, each product 64 bits (`imul`).
pub fn edge_side(point: (u32, u32), from: (u32, u32), to: (u32, u32)) -> Option<(i64, bool)> {
    let ((x, z), (x0, z0), (x1, z1)) = (point, from, to);
    if (x0 ^ x1) | (z0 ^ z1) == 0 {
        return None;
    }
    let d = |a: u32, b: u32| i64::from(a.wrapping_sub(b) as i32);
    let kept = d(z, z0) * d(x1, x0);
    let other = d(x, x0) * d(z1, z0);
    Some((kept, other.wrapping_sub(kept) < 0))
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
        assert_eq!(edge_side((5, 5), (1, 1), (1, 1)), None);
        // The edge along +x from the origin: a point at +z lies on the negative side.
        assert_eq!(edge_side((3, 2), (0, 0), (10, 0)), Some((20, true)));
        assert_eq!(
            edge_side((3, (-2i32) as u32), (0, 0), (10, 0)),
            Some((-20, false))
        );
    }
}
