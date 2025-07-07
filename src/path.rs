use crate::segment::Segment;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[macro_export]
macro_rules! path {
    ($path:literal) => {{
        use std::str::FromStr;
        ::radmin_router::Path::from_str($path).unwrap()
    }};
}

/// A route path, i.e. an ordered list of segments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path(pub Vec<Segment>);

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted = self
            .0
            .iter()
            .map(|seg| match seg {
                Segment::Dynamic(name) => format!("[{}]", name),
                Segment::Literal(segment) => segment.clone(),
            })
            .reduce(|mut acc, v| {
                acc += "/";
                acc += &v;
                acc
            })
            .unwrap_or_default();

        write!(f, "/{}", formatted)
    }
}

impl FromStr for Path {
    type Err = Infallible;

    /// Infallibly parses a `Path` from a string.
    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let path = path.to_string();

        let mut segments = vec![];

        for segment in path.split('/') {
            if segment.is_empty() {
                continue;
            }

            let segment = String::from(segment);

            let chars = segment.chars().collect::<Vec<_>>();
            let is_dynamic = chars[0] == '[' && chars[segment.len() - 1] == ']';

            let segment = if is_dynamic {
                Segment::dynamic(&segment[1..segment.len() - 2])
            } else {
                Segment::literal(&segment)
            };

            segments.push(segment);
        }

        Ok(Path(segments))
    }
}

impl<A> From<A> for Path
where
    A: AsRef<[Segment]>,
{
    fn from(value: A) -> Self {
        Self(value.as_ref().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path() {
        let path = Path(vec![]);
        assert_eq!("/".to_string(), format!("{}", path));
    }

    #[test]
    fn mixed_segment_path() {
        let path = Path(vec![Segment::literal("segment"), Segment::dynamic("slug")]);
        assert_eq!(format!("{}", path), "/segment/[slug]");
    }

    #[test]
    fn path_display() {
        let path = Path(vec![Segment::literal("segment")]);
        assert_eq!(format!("{}", path), "/segment");
    }

    #[test]
    fn path_from_str() {
        let path = Path::from_str("/test/[slug]");
        assert!(path.is_ok());

        let path = path.unwrap();
        assert_eq!(
            path,
            Path(vec![Segment::literal("test"), Segment::dynamic("slug")])
        );

        assert_ne!(
            path,
            Path(vec![
                Segment::literal("another_segment"),
                Segment::dynamic("slug")
            ])
        );
    }
}
