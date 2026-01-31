use std::collections::HashSet;

use crate::model::*;

pub fn print_static(dtd: &Dtd) {
    // Print entities
    let has_entities = !dtd.entities.is_empty();
    if has_entities {
        println!("=== Entities ===");
        for entity in &dtd.entities {
            let prefix = if entity.is_parameter { "%" } else { "&" };
            let suffix = if entity.is_parameter { ";" } else { ";" };
            match &entity.kind {
                EntityKind::Internal { value } => {
                    println!("  {}{}{} = \"{}\"", prefix, entity.name, suffix, value);
                }
                EntityKind::ExternalSystem { uri } => {
                    println!("  {}{}{} = SYSTEM \"{}\"", prefix, entity.name, suffix, uri);
                }
                EntityKind::ExternalPublic { public_id, uri } => {
                    println!(
                        "  {}{}{} = PUBLIC \"{}\" \"{}\"",
                        prefix, entity.name, suffix, public_id, uri
                    );
                }
            }
        }
        println!();
    }

    // Find root elements (not referenced as children by any other element)
    let roots = find_roots(dtd);

    if !roots.is_empty() {
        println!("=== Elements ===");
        let mut expanded = HashSet::new();
        for root in &roots {
            if let Some(elem) = dtd.elements.get(root) {
                let mut ancestors = HashSet::new();
                print_element(dtd, elem, Quantifier::One, "", true, true, &mut ancestors, &mut expanded);
            }
        }
    }
}

fn find_roots(dtd: &Dtd) -> Vec<String> {
    let mut referenced: HashSet<&str> = HashSet::new();
    for elem in dtd.elements.values() {
        collect_child_names(&elem.content, &mut referenced);
    }
    dtd.elements
        .keys()
        .filter(|name| !referenced.contains(name.as_str()))
        .cloned()
        .collect()
}

fn collect_child_names<'a>(content: &'a ContentModel, names: &mut HashSet<&'a str>) {
    match content {
        ContentModel::Children(group) => collect_group_names(group, names),
        ContentModel::Mixed(children) => {
            for name in children {
                names.insert(name);
            }
        }
        _ => {}
    }
}

fn collect_group_names<'a>(group: &'a Group, names: &mut HashSet<&'a str>) {
    for item in &group.items {
        match &item.content {
            GroupItemContent::Name(name) => {
                names.insert(name);
            }
            GroupItemContent::Group(g) => collect_group_names(g, names),
        }
    }
}

fn print_element(dtd: &Dtd, elem: &Element, quantifier: Quantifier, prefix: &str, is_root: bool, is_last: bool, ancestors: &mut HashSet<String>, expanded: &mut HashSet<String>) {
    let connector = if is_root {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    let q = format!("{}", quantifier);
    let attr_str = format_attributes(&elem.attributes);
    let children = get_direct_children(&elem.content);

    // Detect ancestor cycle
    if ancestors.contains(&elem.name) {
        println!("{}{}{}{}{} (recursive)", prefix, connector, elem.name, q, attr_str);
        return;
    }

    // If already expanded elsewhere and has children, show as reference
    if !is_root && expanded.contains(&elem.name) && !children.is_empty() {
        let count = children.len();
        println!("{}{}{}{}{} → ({} children, see above)", prefix, connector, elem.name, q, attr_str, count);
        return;
    }

    println!("{}{}{}{}{}", prefix, connector, elem.name, q, attr_str);

    if children.is_empty() {
        return;
    }

    expanded.insert(elem.name.clone());

    let child_prefix = if is_root {
        String::new()
    } else if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    ancestors.insert(elem.name.clone());

    for (i, (name, child_q)) in children.iter().enumerate() {
        let is_child_last = i == children.len() - 1;
        if let Some(child_elem) = dtd.elements.get(name.as_str()) {
            print_element(dtd, child_elem, *child_q, &child_prefix, false, is_child_last, ancestors, expanded);
        } else {
            let conn = if is_child_last { "└── " } else { "├── " };
            println!("{}{}{}{}", child_prefix, conn, name, child_q);
        }
    }

    ancestors.remove(&elem.name);
}

fn format_attributes(attrs: &[Attribute]) -> String {
    if attrs.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = attrs
        .iter()
        .map(|a| format!("@{}: {} {}", a.name, a.attr_type, a.default))
        .collect();
    format!(" [{}]", parts.join(", "))
}

fn get_direct_children(content: &ContentModel) -> Vec<(String, Quantifier)> {
    match content {
        ContentModel::Children(group) => get_group_children(group),
        ContentModel::Mixed(names) => names.iter().map(|n| (n.clone(), Quantifier::One)).collect(),
        _ => Vec::new(),
    }
}

fn get_group_children(group: &Group) -> Vec<(String, Quantifier)> {
    let mut result = Vec::new();
    for item in &group.items {
        match &item.content {
            GroupItemContent::Name(name) => {
                result.push((name.clone(), item.quantifier));
            }
            GroupItemContent::Group(g) => {
                // Flatten nested groups for display
                result.extend(get_group_children(g));
            }
        }
    }
    result
}
