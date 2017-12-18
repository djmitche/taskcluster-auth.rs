use std::cmp::Ordering;
use std::ops::Index;
use std::fmt;
use std::str;

/// A scope is a bytestring, limited to printable, non-whitespace ASCII characters.  This limit is
/// not currently enforced.
///
/// Scopes sort normally, except that a trailing `*` sorts before its empty prefix; that
/// is,
///   - ab
///   - abc*
///   - abc
///   - abc%
///   - (abc* would appear here in a typical lexical sort)
///   - abc*d
///   - abcd
#[derive(PartialEq, Eq, Clone)]
pub struct Scope(Vec<u8>);

impl From<Vec<u8>> for Scope {
    fn from(vec: Vec<u8>) -> Scope {
        Scope(vec)
    }
}

impl<'a> From<&'a [u8]> for Scope {
    fn from(slice: &'a [u8]) -> Scope {
        Scope(slice.to_vec())
    }
}

impl Index<usize> for Scope {
    type Output = u8;

    fn index(&self, i: usize) -> &u8 {
        &self.0[i]
    }
}

impl Ord for Scope {
    fn cmp(&self, other: &Scope) -> Ordering {
        let a = &self.0;
        let b = &other.0;

        let alen = a.len();
        let astar = alen > 0 && a[alen - 1] == '*' as u8;
        let blen = b.len();
        let bstar = blen > 0 && b[blen - 1] == '*' as u8;

        if !astar && !bstar {
            return a.cmp(&b);
        }
        if astar && bstar {
            let a = &a[0..alen - 1];
            let b = &b[0..blen - 1];
            return a.cmp(&b);
        }
        if astar {
            let a = &a[0..alen - 1];
            if a == &b[..] {
                return Ordering::Less;
            }
            return a.cmp(&b);
        }
        if bstar {
            let b = &b[0..blen - 1];
            if &a[..] == b {
                return Ordering::Greater;
            }
            return a[..].cmp(&b);
        }
        unreachable!();
    }
}

impl PartialOrd for Scope {
    fn partial_cmp(&self, other: &Scope) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "b\"{}\"",
            str::from_utf8(&self.0).map_err(|_| fmt::Error)?
        )
    }
}

impl Scope {
    pub fn new<B: Into<Vec<u8>>>(value: B) -> Scope {
        Scope(value.into())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// "Normalize" a properly sorted set of scopes, eliminating any
/// scopes which are satisfied by other scopes in the set.  For
/// example, if both "abc" and "a*" are included in the input, only
/// "a*" will be returned.
pub fn normalize(mut scopes: Vec<Scope>) -> Vec<Scope> {
    // TODO: don't clone these
    let mut last: Option<Scope> = None;
    let mut last_prefix: Option<Vec<u8>> = None;
    scopes
        .drain(..)
        .filter(|scope| {
            let keep;
            if let Some(ref last) = last {
                // if this is a duplicate, skip it
                if scope == last {
                    keep = false
                } else if let Some(ref last_prefix) = last_prefix {
                    keep = !scope.0.starts_with(&last_prefix)
                } else {
                    keep = true
                }
            } else {
                keep = true
            }

            if keep {
                last = Some(scope.clone());
                if scope.len() > 0 && scope[scope.len() - 1] == '*' as u8 {
                    last_prefix = Some(scope.0[..scope.len() - 1].to_vec());
                } else {
                    last_prefix = None;
                }
            }
            keep
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mkscope(scope: &[u8]) -> Scope {
        scope.to_vec().into()
    }

    fn test_total_order(scopes: Vec<Scope>) {
        for (i, a) in scopes.iter().enumerate() {
            for (j, b) in scopes.iter().enumerate() {
                let exp = i.cmp(&j);
                println!("{:?} {:?} {:?}", a, exp, b);
                assert_eq!(a.cmp(b), exp);
            }
        }
    }

    #[test]
    fn test_stars() {
        test_total_order(vec![
            mkscope(b"*"),
            mkscope(b""),
            mkscope(b"**"),
            mkscope(b"***"),
            mkscope(b"****"),
            mkscope(b"aaa*"),
            mkscope(b"aaa"),
            mkscope(b"aaa**"),
            mkscope(b"aaa***"),
            mkscope(b"aaa****"),
            mkscope(b"aaa*****"),
        ]);
    }

    #[test]
    fn test_all_char_combos() {
        test_total_order(vec![
            mkscope(b"x*"),
            mkscope(b"x"),
            mkscope(b"x%*"),
            mkscope(b"x%"),
            mkscope(b"x%%"),
            mkscope(b"x%a"),
            mkscope(b"x**"),
            mkscope(b"x*%"),
            mkscope(b"x*a"),
            mkscope(b"xa*"),
            mkscope(b"xa"),
            mkscope(b"xa%"),
            mkscope(b"xaa"),
        ]);
    }

    #[test]
    fn sort_star_ordering() {
        let mut scopes: Vec<Scope> = vec![
            mkscope(b"abc"),
            mkscope(b"abc%d"),
            mkscope(b"abc*"),
            mkscope(b"abc*d"),
            mkscope(b"ab"),
            mkscope(b"abcd"),
        ];
        scopes.sort_unstable();
        let expected: Vec<Scope> = vec![
            mkscope(b"ab"),
            mkscope(b"abc*"),
            mkscope(b"abc"),
            mkscope(b"abc%d"),
            mkscope(b"abc*d"),
            mkscope(b"abcd"),
        ];
        assert_eq!(scopes, expected);
    }

    #[test]
    fn normalize_dupes() {
        let scopes: Vec<Scope> = vec![
            mkscope(b"aa"),
            mkscope(b"ab"),
            mkscope(b"ab"),
            mkscope(b"abc"),
            mkscope(b"abc*d"),
        ];
        let expected: Vec<Scope> = vec![
            mkscope(b"aa"),
            mkscope(b"ab"),
            mkscope(b"abc"),
            mkscope(b"abc*d"),
        ];
        let got = normalize(scopes);
        assert_eq!(got, expected);
    }

    #[test]
    fn normalize_stars() {
        let scopes: Vec<Scope> = vec![
            mkscope(b"a*"),
            mkscope(b"ab*"),
            mkscope(b"abcd"),
            mkscope(b"def*"),
            mkscope(b"def"),
            mkscope(b"defghi"),
        ];
        let expected: Vec<Scope> = vec![mkscope(b"a*"), mkscope(b"def*")];
        let got = normalize(scopes);
        assert_eq!(got, expected);
    }
}
