use crate::model::*;

pub fn print_mermaid(dtd: &Dtd) {
    println!("graph TD");
    for elem in dtd.elements.values() {
        let children = get_direct_children(&elem.content);
        for (child_name, quantifier) in children {
            let label = match quantifier {
                Quantifier::One => String::new(),
                q => format!("|\"{}\"| ", q),
            };
            println!("    {} -->{}{}", elem.name, label, child_name);
        }
    }
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
                result.extend(get_group_children(g));
            }
        }
    }
    result
}
