use super::scopes::Scope;
use std::cmp::Ordering;

/// Rule defines an expansion rule.  Scopes matching the given pattern
/// will be expanded using the given scopes (according to all of the rules
/// for kleene stars and parameter expansions).
#[derive(PartialEq, Eq, Clone)]
pub struct Rule {
    pub pattern: Scope,
    pub scopes: Vec<Scope>,
}

impl Ord for Rule {
    fn cmp(&self, other: &Rule) -> Ordering {
        self.pattern.cmp(&other.pattern)
    }
}

impl PartialOrd for Rule {
    fn partial_cmp(&self, other: &Rule) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mkscope(scope: &[u8]) -> Scope {
        scope.to_vec().into()
    }

    #[test]
    fn test_sorting() {
        let mut rules: Vec<Rule> = vec![
            Rule {
                pattern: mkscope(b"def"),
                scopes: vec![mkscope(b"123")],
            },
            Rule {
                pattern: mkscope(b"abc"),
                scopes: vec![],
            },
            Rule {
                pattern: mkscope(b"def*"),
                scopes: vec![],
            },
        ];
        rules.sort_unstable();
        assert_eq!(rules[0].pattern, mkscope(b"abc"));
        assert_eq!(rules[1].pattern, mkscope(b"def*"));
        assert_eq!(rules[2].pattern, mkscope(b"def"));
    }
}
