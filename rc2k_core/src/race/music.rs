//! The race's music at the stage's loading.

/// The race's tune: 6 when the second kind is 1, else 5.
pub fn race_tune(second_kind: u32) -> u32 {
    if second_kind == 1 { 6 } else { 5 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunes() {
        assert_eq!(race_tune(1), 6);
        assert_eq!(race_tune(0), 5);
        assert_eq!(race_tune(2), 5);
    }
}
