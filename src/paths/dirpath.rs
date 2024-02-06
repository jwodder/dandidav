use super::{Component, PurePath};
use derive_more::{AsRef, Deref, Display};
use std::fmt;
use thiserror::Error;

/// A nonempty, forward-slash-separated path that ends in (but does not equal)
/// a forward slash and does not contain any of the following:
///
/// - a `.` or `..` component
/// - a leading forward slash
/// - two or more consecutive forward slashes
/// - NUL
#[derive(AsRef, Clone, Deref, Display, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[as_ref(forward)]
#[deref(forward)]
pub(crate) struct PureDirPath(pub(super) String);

impl PureDirPath {
    pub(crate) fn name_str(&self) -> &str {
        self.0
            .trim_end_matches('/')
            .split('/')
            .next_back()
            .expect("path should be nonempty")
    }

    pub(crate) fn name(&self) -> Component {
        Component(self.name_str().to_owned())
    }

    pub(crate) fn parent(&self) -> Option<PureDirPath> {
        let i = self.0.trim_end_matches('/').rfind('/')?;
        Some(PureDirPath(self.0[..=i].to_owned()))
    }

    pub(crate) fn join(&self, path: &PurePath) -> PurePath {
        PurePath(format!("{self}{path}"))
    }

    pub(crate) fn join_dir(&self, path: &PureDirPath) -> PureDirPath {
        PureDirPath(format!("{self}{path}"))
    }

    pub(crate) fn join_one_dir(&self, c: &Component) -> PureDirPath {
        PureDirPath(format!("{self}{c}/"))
    }

    pub(crate) fn push(&mut self, c: &Component) {
        self.0.push_str(c.as_ref());
        self.0.push('/');
    }

    pub(crate) fn relative_to(&self, dirpath: &PureDirPath) -> Option<PureDirPath> {
        let s = self.0.strip_prefix(&dirpath.0)?;
        (!s.is_empty()).then(|| PureDirPath(s.to_owned()))
    }

    pub(crate) fn component_strs(&self) -> std::str::Split<'_, char> {
        self.0.split('/')
    }
}

impl fmt::Debug for PureDirPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl PartialEq<str> for PureDirPath {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl<'a> PartialEq<&'a str> for PureDirPath {
    fn eq(&self, other: &&'a str) -> bool {
        &self.0 == other
    }
}

impl std::str::FromStr for PureDirPath {
    type Err = ParsePureDirPathError;

    fn from_str(s: &str) -> Result<PureDirPath, ParsePureDirPathError> {
        let Some(pre) = s.strip_suffix('/') else {
            return Err(ParsePureDirPathError::NotDir);
        };
        if s.starts_with('/') {
            Err(ParsePureDirPathError::StartsWithSlash)
        } else if s.contains('\0') {
            Err(ParsePureDirPathError::Nul)
        } else if pre
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
        {
            Err(ParsePureDirPathError::NotNormalized)
        } else {
            Ok(PureDirPath(s.into()))
        }
    }
}

impl From<Component> for PureDirPath {
    fn from(value: Component) -> PureDirPath {
        let mut s = value.0;
        s.push('/');
        PureDirPath(s)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum ParsePureDirPathError {
    #[error("path does not end with a forward slash")]
    NotDir,
    #[error("paths cannot start with a forward slash")]
    StartsWithSlash,
    #[error("paths cannot contain NUL")]
    Nul,
    #[error("path is not normalized")]
    NotNormalized,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use rstest::rstest;

    #[rstest]
    #[case("foo.nwb/")]
    #[case("foo/bar.nwb/")]
    fn test_good_paths(#[case] s: &str) {
        let r = s.parse::<PureDirPath>();
        assert_matches!(r, Ok(_));
    }

    #[rstest]
    #[case("")]
    #[case("/")]
    #[case("/foo")]
    #[case("foo")]
    #[case("foo/bar.nwb")]
    #[case("/foo/")]
    #[case("foo//bar.nwb/")]
    #[case("foo///bar.nwb/")]
    #[case("foo/bar\0.nwb/")]
    #[case("foo/./bar.nwb/")]
    #[case("foo/../bar.nwb/")]
    #[case("./foo/bar.nwb/")]
    #[case("../foo/bar.nwb/")]
    #[case("foo/bar.nwb/.")]
    #[case("foo/bar.nwb/..")]
    #[case("foo/bar.nwb/./")]
    #[case("foo/bar.nwb/../")]
    fn test_bad_paths(#[case] s: &str) {
        let r = s.parse::<PureDirPath>();
        assert_matches!(r, Err(_));
    }

    #[test]
    fn test_parent() {
        let p = "foo/bar/baz/".parse::<PureDirPath>().unwrap();
        assert_matches!(p.parent(), Some(pp) => {
            assert_eq!(pp, "foo/bar/");
        });
    }

    #[test]
    fn test_noparent() {
        let p = "foo/".parse::<PureDirPath>().unwrap();
        assert_matches!(p.parent(), None);
    }
}
