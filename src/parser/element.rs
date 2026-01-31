use anyhow::{bail, Result};

use crate::model::*;
use super::content_model::parse_content_model;

pub fn parse_elements(input: &str) -> Result<Vec<Element>> {
    let mut elements = Vec::new();
    let mut pos = 0;
    let bytes = input.as_bytes();

    while let Some(idx) = find_declaration(input, pos, "<!ELEMENT") {
        pos = idx + 9; // skip "<!ELEMENT"
        // Find the closing '>'
        let end = match find_closing_angle(bytes, pos) {
            Some(e) => e,
            None => bail!("unclosed <!ELEMENT at position {}", idx),
        };
        let body = input[pos..end].trim();
        // body = "name content_model"
        // Split at first whitespace
        let (name, content_str) = split_first_word(body)?;
        let content = parse_content_model(content_str)?;
        elements.push(Element {
            name: name.to_string(),
            content,
            attributes: Vec::new(),
        });
        pos = end + 1;
    }
    Ok(elements)
}

fn split_first_word(s: &str) -> Result<(&str, &str)> {
    let s = s.trim();
    if let Some(i) = s.find(|c: char| c.is_ascii_whitespace()) {
        Ok((s[..i].trim(), s[i..].trim()))
    } else {
        bail!("expected name and content model, got: {}", s);
    }
}

/// Find the start of a declaration like "<!ELEMENT" in the input.
pub(crate) fn find_declaration(input: &str, from: usize, decl: &str) -> Option<usize> {
    input[from..].find(decl).map(|i| from + i)
}

/// Find the matching '>' that closes a declaration, skipping quoted strings.
pub(crate) fn find_closing_angle(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
            }
            b'\'' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
            }
            b'>' => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

