use anyhow::{bail, Result};

use crate::model::*;
use super::element::{find_closing_angle, find_declaration};

pub fn parse_entities(input: &str) -> Result<Vec<Entity>> {
    let mut entities = Vec::new();
    let mut pos = 0;
    let bytes = input.as_bytes();

    while let Some(idx) = find_declaration(input, pos, "<!ENTITY") {
        pos = idx + 8;
        let end = match find_closing_angle(bytes, pos) {
            Some(e) => e,
            None => bail!("unclosed <!ENTITY at position {}", idx),
        };
        let body = input[pos..end].trim();
        if let Some(entity) = parse_entity_body(body)? {
            entities.push(entity);
        }
        pos = end + 1;
    }
    Ok(entities)
}

fn parse_entity_body(body: &str) -> Result<Option<Entity>> {
    let mut tokens = EntityTokenizer::new(body);

    let first = tokens.next_token()?;
    let is_parameter = first == "%";
    let name = if is_parameter {
        tokens.next_token()?
    } else {
        first
    };

    if name.is_empty() {
        return Ok(None);
    }

    let value_or_keyword = tokens.next_token()?;

    let kind = if value_or_keyword.eq_ignore_ascii_case("SYSTEM") {
        let uri = tokens.next_quoted()?;
        EntityKind::ExternalSystem { uri }
    } else if value_or_keyword.eq_ignore_ascii_case("PUBLIC") {
        let public_id = tokens.next_quoted()?;
        let uri = tokens.next_quoted()?;
        EntityKind::ExternalPublic { public_id, uri }
    } else if value_or_keyword.starts_with('"') || value_or_keyword.starts_with('\'') {
        let value = unquote(&value_or_keyword);
        EntityKind::Internal { value }
    } else {
        bail!("unexpected entity value: {}", value_or_keyword);
    };

    Ok(Some(Entity {
        name,
        is_parameter,
        kind,
    }))
}

fn unquote(s: &str) -> String {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Expand parameter entities (%name;) in DTD text.
pub fn expand_parameter_entities(input: &str, entities: &[Entity]) -> String {
    let mut result = input.to_string();
    for entity in entities {
        if entity.is_parameter {
            if let EntityKind::Internal { ref value } = entity.kind {
                let pattern = format!("%{};", entity.name);
                result = result.replace(&pattern, value);
            }
        }
    }
    result
}

struct EntityTokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> EntityTokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn skip_ws(&mut self) {
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() && bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Result<String> {
        self.skip_ws();
        if self.pos >= self.input.len() {
            return Ok(String::new());
        }
        let bytes = self.input.as_bytes();
        if bytes[self.pos] == b'"' || bytes[self.pos] == b'\'' {
            return self.next_quoted_raw();
        }
        let start = self.pos;
        while self.pos < bytes.len() && !bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn next_quoted(&mut self) -> Result<String> {
        let raw = self.next_quoted_raw()?;
        Ok(unquote(&raw))
    }

    fn next_quoted_raw(&mut self) -> Result<String> {
        self.skip_ws();
        let bytes = self.input.as_bytes();
        if self.pos >= bytes.len() {
            bail!("expected quoted string");
        }
        let quote = bytes[self.pos];
        if quote != b'"' && quote != b'\'' {
            bail!("expected quote character");
        }
        let start = self.pos;
        self.pos += 1;
        while self.pos < bytes.len() && bytes[self.pos] != quote {
            self.pos += 1;
        }
        if self.pos >= bytes.len() {
            bail!("unclosed quoted string");
        }
        self.pos += 1;
        Ok(self.input[start..self.pos].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_entity() {
        let input = r#"<!ENTITY copyright "© 2024 Example Corp">"#;
        let entities = parse_entities(input).unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "copyright");
        assert!(!entities[0].is_parameter);
        if let EntityKind::Internal { ref value } = entities[0].kind {
            assert_eq!(value, "© 2024 Example Corp");
        } else {
            panic!("expected internal entity");
        }
    }

    #[test]
    fn test_parameter_entity() {
        let input = r#"<!ENTITY % common "(#PCDATA | em | strong)*">"#;
        let entities = parse_entities(input).unwrap();
        assert_eq!(entities.len(), 1);
        assert!(entities[0].is_parameter);
        assert_eq!(entities[0].name, "common");
    }

    #[test]
    fn test_expand() {
        let entities = vec![Entity {
            name: "common".to_string(),
            is_parameter: true,
            kind: EntityKind::Internal {
                value: "(#PCDATA | em)*".to_string(),
            },
        }];
        let result = expand_parameter_entities("<!ELEMENT p %common;>", &entities);
        assert_eq!(result, "<!ELEMENT p (#PCDATA | em)*>");
    }
}
