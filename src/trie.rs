use super::scopes::{Scope, normalize};
use super::rules::Rule;
use std::fmt;
use std::str;

struct Node {
    /// Scopes to apply when traversing this node
    scopes: Vec<Scope>,

    /// Next state for each possible next byte in the input
    next: [Option<Box<Node>>; 256],

    /// Node if this is the last character in the input
    end: Option<Box<Node>>,
}

impl Node {
    fn new() -> Node {
        Node {
            scopes: vec![],
            // Option<Box<..>> does not implement Copy, so rust can't figure out how to initialize
            // this with `[None, 256]` :(
            next: [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            end: None,
        }
    }

    fn build(mut rules: Vec<Rule>) -> Node {
        // ensure the rules are in shape (sorted, with sorted, normalized scopes)
        rules.sort_unstable();
        let rules: Vec<Rule> = rules
            .drain(..)
            .map(|mut rule| {
                rule.scopes.sort_unstable();
                rule.scopes = normalize(rule.scopes);
                rule
            })
            .collect();

        // TODO: avoid cloning

        // generate the state for the slice of rules, starting at character position k in each.
        // This assumes that the rules have the same value at positions < k, as assured by the sort
        // order.  Thus we have three segments to work with, in order:
        //         k
        //         |
        //      ...*   -- (A) terminal * in pattern
        //      ...    -- (B) end of pattern (where pattern[k-1] != '*')
        //      ...*a  \
        //      ...a   -- (C) nonterminal * or other characters in pattern
        //      ...ab  /
        fn gen(rules: &[Rule], k: usize) -> Node {
            let mut node = Node::new();
            let mut j = 0;
            let n = rules.len();
            while j < n {
                let pattern = &rules[j].pattern;
                if k == pattern.len() {
                    // case B
                    node.end = Some(Box::new(Node::new()));
                    let mut endstate = Box::new(Node::new());
                    endstate.scopes = rules[j].scopes.clone();
                    node.end = Some(endstate);
                    j += 1;
                    continue;
                }

                let current = pattern[k];
                if current == b'*' && pattern.len() == k + 1 {
                    // case A
                    node.scopes = rules[j].scopes.clone();
                    j += 1;
                } else {
                    // case C
                    let seg = j;
                    loop {
                        j += 1;
                        if j >= n || rules[j].pattern[k] != current {
                            break;
                        }
                    }
                    node.next[current as usize] = Some(Box::new(gen(&rules[seg..j], k + 1)));
                }
            }

            // a terminal star in the input here (state['*']['end']) will match all
            // patterns, but we have already matched the `*` patterns (case A), so we
            // can skip those.
            let mut i = 0;
            while i < n && rules[i].pattern.len() == k + 1 && rules[i].pattern[k] == '*' as u8 {
                i += 1;
            }
            if i < n {
                let all_scopes: Vec<Scope> = rules[i..]
                    .iter()
                    .map(|rule| rule.scopes.clone())
                    .flat_map(|s| s)
                    .collect();
                match node.next['*' as usize] {
                    Some(ref mut subnode) => {
                        subnode.scopes = all_scopes;
                    }
                    None => {
                        let mut subnode = Node::new();
                        subnode.scopes = all_scopes;
                        node.next['*' as usize] = Some(Box::new(subnode));
                    }
                };
            }

            node
        };
        gen(&rules, 0)
    }

    fn debug_fmt(&self, f: &mut fmt::Formatter, indent: &str) -> fmt::Result {
        write!(f, "{}Node {{\n", indent)?;
        write!(f, "{}  scopes: {:?}", indent, self.scopes)?;
        for i in 0..256 {
            if let Some(ref node) = self.next[i] {
                match str::from_utf8(&vec![i as u8]) {
                    Ok(c) => write!(f, ",\n{}  next[{:?}]:\n", indent, c)?,
                    Err(_) => write!(f, ",\n{}  next[{:?}]:\n", indent, i)?,
                }
                node.debug_fmt(f, &format!("{}    ", indent))?;
            }
        }
        if let Some(ref node) = self.end {
            write!(f, ",\n{}  end:\n", indent)?;
            node.debug_fmt(f, &format!("{}    ", indent))?;
        }
        write!(f, "\n{}}}", indent)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f, "")
    }
}

pub struct Trie {
    start: Node,
}

impl Trie {
    pub fn new(rules: Vec<Rule>) -> Trie {
        Trie { start: Node::build(rules) }
    }
}

impl fmt::Debug for Trie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Trie {{\n  start:\n")?;
        self.start.debug_fmt(f, "    ")?;
        write!(f, "\n}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mkscope(scope: &[u8]) -> Scope {
        scope.to_vec().into()
    }

    #[test]
    fn constructor() {
        let mut rules: Vec<Rule> = vec![
            Rule {
                pattern: mkscope(b"a"),
                scopes: vec![mkscope(b"x"), mkscope(b"y")],
            },
        ];
        assert_eq!(
            format!("{:?}", Trie::new(rules)),
            r###"Trie {
  start:
    Node {
      scopes: [],
      next["*"]:
        Node {
          scopes: [b"x", b"y"]
        },
      next["a"]:
        Node {
          scopes: [],
          next["*"]:
            Node {
              scopes: [b"x", b"y"]
            },
          end:
            Node {
              scopes: [b"x", b"y"]
            }
        }
    }
}"###
        );
    }
}
