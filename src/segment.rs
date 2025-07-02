use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

/// A URL path segment. All dynamic segments are equal regardless
/// of their name.
#[derive(Clone)]
pub enum Segment {
    /// A fixed, literal path segment that is matched exactly.
    Literal(String),
    /// A dynamic path segment that matches any single segment.
    /// All dynamic segments are considered equal regardless of
    /// their name.
    ///
    /// `/some/(dynamic)/segment` matches
    /// - `/some/cool/segment` and
    /// - `/some/unknown/segment`
    Dynamic(String),
}

impl Segment {
    /// Constructs a literal segment from any `Into<String>`
    pub fn literal(literal: impl Into<String>) -> Self {
        Self::Literal(literal.into())
    }

    /// Constructs a dynamic segment from any `Into<String>
    pub fn dynamic(dynamic: impl Into<String>) -> Self {
        Self::Dynamic(dynamic.into())
    }
}

impl Debug for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dynamic(name) => write!(f, "[{}]", name),
            Self::Literal(segment) => write!(f, "{}", segment),
        }
    }
}

impl Hash for Segment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Self::Literal(literal) = self {
            literal.hash(state);
        }
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Segment::Literal(lhs), Segment::Literal(rhs)) => lhs == rhs,
            (Segment::Dynamic(_), Segment::Dynamic(_)) => true,
            _ => false,
        }
    }
}

impl Eq for Segment {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::DefaultHasher;

    fn hash(segment: &Segment) -> u64 {
        let mut hasher = DefaultHasher::new();
        segment.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn hash_literal() {
        let segment1a = Segment::literal("1");
        let segment1b = Segment::literal("1");
        assert_eq!(hash(&segment1a), hash(&segment1b));

        let segment2 = Segment::literal("2");
        assert_ne!(hash(&segment1a), hash(&segment2));
    }

    #[test]
    fn hash_dynamic() {
        let segment1a = Segment::dynamic("1");
        let segment1b = Segment::dynamic("1");
        assert_eq!(hash(&segment1a), hash(&segment1b));

        let segment2 = Segment::dynamic("2");
        assert_eq!(hash(&segment1a), hash(&segment2));
    }
}
