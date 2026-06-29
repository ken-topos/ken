//! `unknown` propagation — `spec/40-runtime/41-values.md §6`.
//!
//! `unknown` is the third truth value: the operational residue of an open
//! verification hole.  It propagates under Kleene/Heyting logic:
//!   unknown ∧ false = false   (false dominates AND)
//!   unknown ∨ true  = true    (true  dominates OR)
//!   otherwise       = unknown

/// Three-valued logic value for unknown propagation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unknown {
    False,
    True,
    Unknown,
}

impl Unknown {
    /// Kleene/Heyting conjunction.
    pub fn and(self, other: Unknown) -> Unknown {
        match (self, other) {
            (Unknown::False, _) | (_, Unknown::False) => Unknown::False,
            (Unknown::True, Unknown::True) => Unknown::True,
            _ => Unknown::Unknown,
        }
    }

    /// Kleene/Heyting disjunction.
    pub fn or(self, other: Unknown) -> Unknown {
        match (self, other) {
            (Unknown::True, _) | (_, Unknown::True) => Unknown::True,
            (Unknown::False, Unknown::False) => Unknown::False,
            _ => Unknown::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Unknown::*;

    // --- conformance: runtime/values/unknown-propagates ---
    #[test]
    fn unknown_and_false_is_false() {
        assert_eq!(Unknown.and(False), False);
        assert_eq!(False.and(Unknown), False);
    }

    #[test]
    fn unknown_or_true_is_true() {
        assert_eq!(Unknown.or(True), True);
        assert_eq!(True.or(Unknown), True);
    }

    #[test]
    fn unknown_and_true_is_unknown() {
        assert_eq!(Unknown.and(True), Unknown);
        assert_eq!(True.and(Unknown), Unknown);
    }

    #[test]
    fn unknown_or_false_is_unknown() {
        assert_eq!(Unknown.or(False), Unknown);
        assert_eq!(False.or(Unknown), Unknown);
    }

    #[test]
    fn unknown_and_unknown_is_unknown() {
        assert_eq!(Unknown.and(Unknown), Unknown);
    }

    #[test]
    fn unknown_or_unknown_is_unknown() {
        assert_eq!(Unknown.or(Unknown), Unknown);
    }

    #[test]
    fn standard_boolean_cases_hold() {
        assert_eq!(True.and(True), True);
        assert_eq!(True.and(False), False);
        assert_eq!(False.or(False), False);
        assert_eq!(True.or(False), True);
    }
}
