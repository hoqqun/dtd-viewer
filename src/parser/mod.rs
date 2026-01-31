pub mod attlist;
pub mod content_model;
pub mod element;
pub mod entity;

use anyhow::Result;
use indexmap::IndexMap;

use crate::model::*;

pub fn parse_dtd(input: &str) -> Result<Dtd> {
    // 1. Parse entities first
    let entities = entity::parse_entities(input)?;

    // 2. Expand parameter entities in the source
    let expanded = entity::expand_parameter_entities(input, &entities);

    // 3. Parse elements
    let elements = element::parse_elements(&expanded)?;

    // 4. Parse attlists
    let attlists = attlist::parse_attlists(&expanded)?;

    // 5. Build element map
    let mut element_map: IndexMap<String, Element> = IndexMap::new();
    for elem in elements {
        element_map.insert(elem.name.clone(), elem);
    }

    // 6. Attach attributes to elements
    for (elem_name, attrs) in attlists {
        if let Some(elem) = element_map.get_mut(&elem_name) {
            elem.attributes.extend(attrs);
        } else {
            // Create element stub for attlist without matching element
            element_map.insert(
                elem_name.clone(),
                Element {
                    name: elem_name,
                    content: ContentModel::Any,
                    attributes: attrs,
                },
            );
        }
    }

    Ok(Dtd {
        elements: element_map,
        entities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_parse() {
        let input = r#"
<!ENTITY % inline "(#PCDATA | em)*">

<!ELEMENT library (book+)>
<!ELEMENT book (title, author?, chapter+)>
<!ELEMENT title %inline;>
<!ELEMENT author %inline;>
<!ELEMENT chapter (heading, paragraph*)>
<!ELEMENT heading (#PCDATA)>
<!ELEMENT paragraph %inline;>
<!ELEMENT em (#PCDATA)>

<!ATTLIST book
    id ID #REQUIRED
    lang CDATA #IMPLIED
>
        "#;

        let dtd = parse_dtd(input).unwrap();
        assert_eq!(dtd.elements.len(), 8);
        assert!(dtd.elements.contains_key("library"));
        assert!(dtd.elements.contains_key("book"));

        let book = &dtd.elements["book"];
        assert_eq!(book.attributes.len(), 2);
        assert_eq!(book.attributes[0].name, "id");

        assert_eq!(dtd.entities.len(), 1);
        assert!(dtd.entities[0].is_parameter);
    }
}
