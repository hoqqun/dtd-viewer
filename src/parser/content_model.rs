use anyhow::{bail, Result};

use crate::model::*;

pub fn parse_content_model(input: &str) -> Result<ContentModel> {
    let s = input.trim();
    if s.eq_ignore_ascii_case("EMPTY") {
        return Ok(ContentModel::Empty);
    }
    if s.eq_ignore_ascii_case("ANY") {
        return Ok(ContentModel::Any);
    }
    let mut p = Parser::new(s);
    let model = p.parse_top()?;
    Ok(model)
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn expect(&mut self, ch: u8) -> Result<()> {
        self.skip_ws();
        if self.peek() == Some(ch) {
            self.pos += 1;
            Ok(())
        } else {
            bail!(
                "expected '{}' at position {}, got {:?}",
                ch as char,
                self.pos,
                self.peek().map(|b| b as char)
            );
        }
    }

    fn parse_top(&mut self) -> Result<ContentModel> {
        self.skip_ws();
        if self.peek() != Some(b'(') {
            bail!("expected '(' at start of content model");
        }

        if self.is_mixed() {
            return self.parse_mixed();
        }

        let group = self.parse_group()?;
        let q = self.parse_quantifier();
        if q != Quantifier::One {
            // Wrap in a single-item group with the quantifier
            Ok(ContentModel::Children(Group {
                kind: GroupKind::Sequence,
                items: vec![GroupItem {
                    content: GroupItemContent::Group(group),
                    quantifier: q,
                }],
            }))
        } else {
            Ok(ContentModel::Children(group))
        }
    }

    fn is_mixed(&self) -> bool {
        let rest = std::str::from_utf8(&self.input[self.pos..]).unwrap_or("");
        let trimmed = rest.trim_start_matches('(').trim_start();
        trimmed.starts_with("#PCDATA")
    }

    fn parse_mixed(&mut self) -> Result<ContentModel> {
        self.expect(b'(')?;
        self.skip_ws();
        let name = self.parse_name()?;
        if name != "#PCDATA" {
            bail!("expected #PCDATA in mixed content");
        }
        self.skip_ws();
        if self.peek() == Some(b')') {
            self.pos += 1;
            if self.peek() == Some(b'*') {
                self.pos += 1;
            }
            return Ok(ContentModel::Pcdata);
        }

        let mut names = Vec::new();
        while self.peek() == Some(b'|') {
            self.pos += 1;
            self.skip_ws();
            names.push(self.parse_name()?);
            self.skip_ws();
        }
        self.expect(b')')?;
        if self.peek() == Some(b'*') {
            self.pos += 1;
        }
        Ok(ContentModel::Mixed(names))
    }

    /// Parse a parenthesized group (without outer quantifier).
    fn parse_group(&mut self) -> Result<Group> {
        self.expect(b'(')?;
        self.skip_ws();

        let first = self.parse_cp()?;
        self.skip_ws();

        match self.peek() {
            Some(b')') => {
                self.pos += 1;
                Ok(Group {
                    kind: GroupKind::Sequence,
                    items: vec![first],
                })
            }
            Some(b',') => {
                let mut items = vec![first];
                while self.peek() == Some(b',') {
                    self.pos += 1;
                    self.skip_ws();
                    items.push(self.parse_cp()?);
                    self.skip_ws();
                }
                self.expect(b')')?;
                Ok(Group {
                    kind: GroupKind::Sequence,
                    items,
                })
            }
            Some(b'|') => {
                let mut items = vec![first];
                while self.peek() == Some(b'|') {
                    self.pos += 1;
                    self.skip_ws();
                    items.push(self.parse_cp()?);
                    self.skip_ws();
                }
                self.expect(b')')?;
                Ok(Group {
                    kind: GroupKind::Choice,
                    items,
                })
            }
            other => bail!("unexpected {:?} in group", other.map(|b| b as char)),
        }
    }

    /// Parse a content particle: name or nested group, followed by optional quantifier.
    fn parse_cp(&mut self) -> Result<GroupItem> {
        self.skip_ws();
        let content = if self.peek() == Some(b'(') {
            let group = self.parse_group()?;
            GroupItemContent::Group(group)
        } else {
            let name = self.parse_name()?;
            GroupItemContent::Name(name)
        };
        let quantifier = self.parse_quantifier();
        Ok(GroupItem {
            content,
            quantifier,
        })
    }

    fn parse_quantifier(&mut self) -> Quantifier {
        match self.peek() {
            Some(b'?') => {
                self.pos += 1;
                Quantifier::Optional
            }
            Some(b'*') => {
                self.pos += 1;
                Quantifier::ZeroOrMore
            }
            Some(b'+') => {
                self.pos += 1;
                Quantifier::OneOrMore
            }
            _ => Quantifier::One,
        }
    }

    fn parse_name(&mut self) -> Result<String> {
        self.skip_ws();
        let start = self.pos;
        if self.pos < self.input.len() && self.input[self.pos] == b'#' {
            self.pos += 1;
        }
        while self.pos < self.input.len()
            && (self.input[self.pos].is_ascii_alphanumeric()
                || self.input[self.pos] == b'-'
                || self.input[self.pos] == b'_'
                || self.input[self.pos] == b'.'
                || self.input[self.pos] == b':')
        {
            self.pos += 1;
        }
        if self.pos == start {
            bail!("expected name at position {}", self.pos);
        }
        Ok(String::from_utf8_lossy(&self.input[start..self.pos]).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert!(matches!(parse_content_model("EMPTY").unwrap(), ContentModel::Empty));
    }

    #[test]
    fn test_any() {
        assert!(matches!(parse_content_model("ANY").unwrap(), ContentModel::Any));
    }

    #[test]
    fn test_pcdata() {
        assert!(matches!(parse_content_model("(#PCDATA)").unwrap(), ContentModel::Pcdata));
    }

    #[test]
    fn test_pcdata_star() {
        assert!(matches!(parse_content_model("(#PCDATA)*").unwrap(), ContentModel::Pcdata));
    }

    #[test]
    fn test_mixed() {
        if let ContentModel::Mixed(names) = parse_content_model("(#PCDATA | em | strong)*").unwrap() {
            assert_eq!(names, vec!["em", "strong"]);
        } else {
            panic!("expected Mixed");
        }
    }

    #[test]
    fn test_sequence() {
        if let ContentModel::Children(g) = parse_content_model("(title, author, year)").unwrap() {
            assert_eq!(g.kind, GroupKind::Sequence);
            assert_eq!(g.items.len(), 3);
        } else {
            panic!("expected Children");
        }
    }

    #[test]
    fn test_choice() {
        if let ContentModel::Children(g) = parse_content_model("(a | b | c)").unwrap() {
            assert_eq!(g.kind, GroupKind::Choice);
            assert_eq!(g.items.len(), 3);
        } else {
            panic!("expected Children");
        }
    }

    #[test]
    fn test_quantifiers() {
        if let ContentModel::Children(g) = parse_content_model("(a+, b?, c*)").unwrap() {
            assert_eq!(g.items[0].quantifier, Quantifier::OneOrMore);
            assert_eq!(g.items[1].quantifier, Quantifier::Optional);
            assert_eq!(g.items[2].quantifier, Quantifier::ZeroOrMore);
        } else {
            panic!("expected Children");
        }
    }

    #[test]
    fn test_nested_group() {
        let m = parse_content_model("(a, (b | c)+)").unwrap();
        if let ContentModel::Children(g) = m {
            assert_eq!(g.kind, GroupKind::Sequence);
            assert_eq!(g.items.len(), 2);
            assert_eq!(g.items[1].quantifier, Quantifier::OneOrMore);
            if let GroupItemContent::Group(ref inner) = g.items[1].content {
                assert_eq!(inner.kind, GroupKind::Choice);
                assert_eq!(inner.items.len(), 2);
            } else {
                panic!("expected nested group");
            }
        } else {
            panic!("expected Children");
        }
    }
}
