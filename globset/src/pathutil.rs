use std::borrow::Cow;

use bstr::BStr;

/// The final component of the path, if it is a normal file.
///
/// If the path terminates in ., .., or consists solely of a root of prefix,
/// file_name will return None.
pub fn file_name<'a>(path: &Cow<'a, BStr>) -> Option<Cow<'a, BStr>> {
    if path.is_empty() {
        return None;
    } else if path.len() == 1 && path[0] == b'.' {
        return None;
    } else if path.last() == Some(b'.') {
        return None;
    } else if path.len() >= 2 && &path[path.len() - 2..] == ".." {
        return None;
    }
    let last_slash = path.rfind_byte(b'/').map(|i| i + 1).unwrap_or(0);
    Some(match *path {
        Cow::Borrowed(path) => Cow::Borrowed(&path[last_slash..]),
        Cow::Owned(ref path) => {
            let mut path = path.clone();
            path.drain_bytes(..last_slash);
            Cow::Owned(path)
        }
    })
}

/// Return a file extension given a path's file name.
///
/// Note that this does NOT match the semantics of std::path::Path::extension.
/// Namely, the extension includes the `.` and matching is otherwise more
/// liberal. Specifically, the extenion is:
///
/// * None, if the file name given is empty;
/// * None, if there is no embedded `.`;
/// * Otherwise, the portion of the file name starting with the final `.`.
///
/// e.g., A file name of `.rs` has an extension `.rs`.
///
/// N.B. This is done to make certain glob match optimizations easier. Namely,
/// a pattern like `*.rs` is obviously trying to match files with a `rs`
/// extension, but it also matches files like `.rs`, which doesn't have an
/// extension according to std::path::Path::extension.
pub fn file_name_ext<'a>(name: &Cow<'a, BStr>) -> Option<Cow<'a, BStr>> {
    if name.is_empty() {
        return None;
    }
    let last_dot_at = {
        let result = name
            .bytes().enumerate().rev()
            .find(|&(_, b)| b == b'.')
            .map(|(i, _)| i);
        match result {
            None => return None,
            Some(i) => i,
        }
    };
    Some(match *name {
        Cow::Borrowed(name) => Cow::Borrowed(&name[last_dot_at..]),
        Cow::Owned(ref name) => {
            let mut name = name.clone();
            name.drain_bytes(..last_dot_at);
            Cow::Owned(name)
        }
    })
}

/// Normalizes a path to use `/` as a separator everywhere, even on platforms
/// that recognize other characters as separators.
#[cfg(unix)]
pub fn normalize_path(path: Cow<BStr>) -> Cow<BStr> {
    // UNIX only uses /, so we're good.
    path
}

/// Normalizes a path to use `/` as a separator everywhere, even on platforms
/// that recognize other characters as separators.
#[cfg(not(unix))]
pub fn normalize_path(mut path: Cow<BStr>) -> Cow<BStr> {
    use std::path::is_separator;

    for i in 0..path.len() {
        if path[i] == b'/' || !is_separator(path[i] as char) {
            continue;
        }
        path.to_mut()[i] = b'/';
    }
    path
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use bstr::{B, BString};

    use super::{file_name_ext, normalize_path};

    macro_rules! ext {
        ($name:ident, $file_name:expr, $ext:expr) => {
            #[test]
            fn $name() {
                let bs = BString::from($file_name);
                let got = file_name_ext(&Cow::Owned(bs));
                assert_eq!($ext.map(|s| Cow::Borrowed(B(s))), got);
            }
        };
    }

    ext!(ext1, "foo.rs", Some(".rs"));
    ext!(ext2, ".rs", Some(".rs"));
    ext!(ext3, "..rs", Some(".rs"));
    ext!(ext4, "", None::<&str>);
    ext!(ext5, "foo", None::<&str>);

    macro_rules! normalize {
        ($name:ident, $path:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let bs = BString::from_slice($path);
                let got = normalize_path(Cow::Owned(bs));
                assert_eq!($expected.to_vec(), got.into_owned());
            }
        };
    }

    normalize!(normal1, b"foo", b"foo");
    normalize!(normal2, b"foo/bar", b"foo/bar");
    #[cfg(unix)]
    normalize!(normal3, b"foo\\bar", b"foo\\bar");
    #[cfg(not(unix))]
    normalize!(normal3, b"foo\\bar", b"foo/bar");
    #[cfg(unix)]
    normalize!(normal4, b"foo\\bar/baz", b"foo\\bar/baz");
    #[cfg(not(unix))]
    normalize!(normal4, b"foo\\bar/baz", b"foo/bar/baz");
}
