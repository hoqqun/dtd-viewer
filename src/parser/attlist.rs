use anyhow::{bail, Result};

use crate::model::*;
use super::element::{find_closing_angle, find_declaration};

/// Returns Vec<(element_name, Vec<Attribute>)>
pub fn parse_attlists(input: &str) -> Result<Vec<(String, Vec<Attribute>)>> {
    let mut result = Vec::new();
    let mut pos = 0;
    let bytes = input.as_bytes();

    while let Some(idx) = find_declaration(input, pos, "<!ATTLIST") {
        pos = idx + 9;
        let end = match find_closing_angle(bytes, pos) {
            Some(e) => e,
            None => bail!("unclosed <!ATTLIST at position {}", idx),
        };
        let body = input[pos..end].trim();
        let (elem_name, attrs) = parse_attlist_body(body)?;
        result.push((elem_name, attrs));
        pos = end + 1;
    }
    Ok(result)
}

fn parse_attlist_body(body: &str) -> Result<(String, Vec<Attribute>)> {
    let mut p = AttParser::new(body);
    let elem_name = p.read_name()?;
    let mut attrs = Vec::new();

    while p.has_more() {
        if let Some(attr) = p.read_attribute()? {
            attrs.push(attr);
        } else {
            break;
        }
    }

    Ok((elem_name, attrs))
}

struct AttParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> AttParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn skip_ws(&mut self) {
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() && bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn has_more(&self) -> bool {
        self.pos < self.input.len()
            && self.input[self.pos..].trim_start().len() > 0
    }

    fn read_name(&mut self) -> Result<String> {
        self.skip_ws();
        let bytes = self.input.as_bytes();
        let start = self.pos;
        while self.pos < bytes.len()
            && (bytes[self.pos].is_ascii_alphanumeric()
                || bytes[self.pos] == b'-'
                || bytes[self.pos] == b'_'
                || bytes[self.pos] == b':'
                || bytes[self.pos] == b'.')
        {
            self.pos += 1;
        }
        if self.pos == start {
            bail!("expected name at pos {}", self.pos);
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn read_attribute(&mut self) -> Result<Option<Attribute>> {
        self.skip_ws();
        if !self.has_more() {
            return Ok(None);
        }

        let name = self.read_name()?;
        let attr_type = self.read_attr_type()?;
        let default = self.read_attr_default()?;

        Ok(Some(Attribute {
            name,
            attr_type,
            default,
        }))
    }

    fn read_attr_type(&mut self) -> Result<AttrType> {
        self.skip_ws();
        let bytes = self.input.as_bytes();

        // Check for enumeration: (a|b|c)
        if self.pos < bytes.len() && bytes[self.pos] == b'(' {
            return self.read_enumeration();
        }

        let keyword = self.read_name()?;
        match keyword.to_uppercase().as_str() {
            "CDATA" => Ok(AttrType::Cdata),
            "ID" => Ok(AttrType::Id),
            "IDREF" => Ok(AttrType::Idref),
            "IDREFS" => Ok(AttrType::Idrefs),
            "NMTOKEN" => Ok(AttrType::Nmtoken),
            "NMTOKENS" => Ok(AttrType::Nmtokens),
            other => bail!("unknown attribute type: {}", other),
        }
    }

    fn read_enumeration(&mut self) -> Result<AttrType> {
        let bytes = self.input.as_bytes();
        self.pos += 1; // skip '('
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            let name = self.read_name()?;
            values.push(name);
            self.skip_ws();
            if self.pos < bytes.len() && bytes[self.pos] == b'|' {
                self.pos += 1;
            } else if self.pos < bytes.len() && bytes[self.pos] == b')' {
                self.pos += 1;
                break;
            } else {
                bail!("expected '|' or ')' in enumeration");
            }
        }
        Ok(AttrType::Enumeration(values))
    }

    fn read_attr_default(&mut self) -> Result<AttrDefault> {
        self.skip_ws();
        let bytes = self.input.as_bytes();

        if self.pos < bytes.len() && bytes[self.pos] == b'#' {
            // #REQUIRED, #IMPLIED, #FIXED
            self.pos += 1;
            let keyword = self.read_name()?;
            match keyword.to_uppercase().as_str() {
                "REQUIRED" => Ok(AttrDefault::Required),
                "IMPLIED" => Ok(AttrDefault::Implied),
                "FIXED" => {
                    let val = self.read_quoted()?;
                    Ok(AttrDefault::Fixed(val))
                }
                other => bail!("unknown default: #{}", other),
            }
        } else if self.pos < bytes.len() && (bytes[self.pos] == b'"' || bytes[self.pos] == b'\'') {
            let val = self.read_quoted()?;
            Ok(AttrDefault::Default(val))
        } else {
            bail!("expected attribute default at pos {}", self.pos);
        }
    }

    fn read_quoted(&mut self) -> Result<String> {
        self.skip_ws();
        let bytes = self.input.as_bytes();
        if self.pos >= bytes.len() {
            bail!("expected quoted string");
        }
        let quote = bytes[self.pos];
        if quote != b'"' && quote != b'\'' {
            bail!("expected quote");
        }
        self.pos += 1;
        let start = self.pos;
        while self.pos < bytes.len() && bytes[self.pos] != quote {
            self.pos += 1;
        }
        if self.pos >= bytes.len() {
            bail!("unclosed quote");
        }
        let value = self.input[start..self.pos].to_string();
        self.pos += 1;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_attr() {
        let input = r#"<!ATTLIST book id ID #REQUIRED>"#;
        let result = parse_attlists(input).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "book");
        assert_eq!(result[0].1.len(), 1);
        assert_eq!(result[0].1[0].name, "id");
        assert!(matches!(result[0].1[0].attr_type, AttrType::Id));
        assert!(matches!(result[0].1[0].default, AttrDefault::Required));
    }

    #[test]
    fn test_multiple_attrs() {
        let input = r#"<!ATTLIST book
            id ID #REQUIRED
            lang CDATA #IMPLIED
        >"#;
        let result = parse_attlists(input).unwrap();
        assert_eq!(result[0].1.len(), 2);
    }

    #[test]
    fn test_enumeration() {
        let input = r#"<!ATTLIST item type (a|b|c) "a">"#;
        let result = parse_attlists(input).unwrap();
        if let AttrType::Enumeration(ref vals) = result[0].1[0].attr_type {
            assert_eq!(vals, &["a", "b", "c"]);
        } else {
            panic!("expected enumeration");
        }
        if let AttrDefault::Default(ref v) = result[0].1[0].default {
            assert_eq!(v, "a");
        } else {
            panic!("expected default");
        }
    }
}
