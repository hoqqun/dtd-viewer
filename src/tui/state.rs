use std::collections::HashSet;

use crate::model::*;

#[derive(Debug, Clone)]
pub struct TreeRow {
    pub depth: usize,
    pub element_name: String,
    pub quantifier: Quantifier,
    pub has_children: bool,
    pub is_expanded: bool,
    pub path: String, // unique path like "library/book/chapter"
}

#[derive(Debug, Clone)]
pub enum Overlay {
    Entities,
    Attributes(String), // element name
}

pub struct AppState {
    pub dtd: Dtd,
    pub rows: Vec<TreeRow>,
    pub cursor: usize,
    pub expanded: HashSet<String>,
    pub search_mode: bool,
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub search_match_idx: Option<usize>,
    pub overlay: Option<Overlay>,
}

impl AppState {
    pub fn new(dtd: Dtd) -> Self {
        let roots = find_roots(&dtd);
        let mut expanded = HashSet::new();
        // Expand roots by default
        for root in &roots {
            expanded.insert(root.clone());
        }

        let mut state = Self {
            dtd,
            rows: Vec::new(),
            cursor: 0,
            expanded,
            search_mode: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_match_idx: None,
            overlay: None,
        };
        state.rebuild_rows();
        state
    }

    pub fn rebuild_rows(&mut self) {
        self.rows.clear();
        let roots = find_roots(&self.dtd);
        for root in roots {
            let mut ancestors = HashSet::new();
            self.build_subtree(&root, 0, &root, Quantifier::One, &mut ancestors);
        }
    }

    fn build_subtree(&mut self, elem_name: &str, depth: usize, path: &str, quantifier: Quantifier, ancestors: &mut HashSet<String>) {
        let is_cycle = ancestors.contains(elem_name);

        let has_children = if is_cycle {
            false
        } else {
            self.dtd
                .elements
                .get(elem_name)
                .map(|e| !get_direct_children(&e.content).is_empty())
                .unwrap_or(false)
        };

        let is_expanded = has_children && self.expanded.contains(path);

        self.rows.push(TreeRow {
            depth,
            element_name: elem_name.to_string(),
            quantifier,
            has_children,
            is_expanded,
            path: path.to_string(),
        });

        if is_expanded {
            ancestors.insert(elem_name.to_string());
            if let Some(elem) = self.dtd.elements.get(elem_name).cloned() {
                let children = get_direct_children(&elem.content);
                for (child_name, child_q) in children {
                    let child_path = format!("{}/{}", path, child_name);
                    self.build_subtree(&child_name, depth + 1, &child_path, child_q, ancestors);
                }
            }
            ancestors.remove(elem_name);
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.rows.len() {
            self.cursor += 1;
        }
    }

    pub fn expand(&mut self) {
        if let Some(row) = self.rows.get(self.cursor) {
            if row.has_children {
                self.expanded.insert(row.path.clone());
                self.rebuild_rows();
            }
        }
    }

    pub fn collapse(&mut self) {
        if let Some(row) = self.rows.get(self.cursor) {
            if row.is_expanded {
                self.expanded.remove(&row.path);
                self.rebuild_rows();
            } else if row.depth > 0 {
                // Move to parent
                let parent_depth = row.depth - 1;
                for i in (0..self.cursor).rev() {
                    if self.rows[i].depth == parent_depth {
                        self.cursor = i;
                        break;
                    }
                }
            }
        }
    }

    pub fn start_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_matches.clear();
        self.search_match_idx = None;
    }

    pub fn cancel_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_matches.clear();
        self.search_match_idx = None;
    }

    pub fn finish_search(&mut self) {
        self.search_mode = false;
        self.update_search_matches();
        if !self.search_matches.is_empty() {
            self.search_match_idx = Some(0);
            self.cursor = self.search_matches[0];
        }
    }

    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.update_search_matches();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.update_search_matches();
    }

    fn update_search_matches(&mut self) {
        let q = self.search_query.to_lowercase();
        self.search_matches = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.element_name.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
    }

    pub fn next_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        let idx = match self.search_match_idx {
            Some(i) => (i + 1) % self.search_matches.len(),
            None => 0,
        };
        self.search_match_idx = Some(idx);
        self.cursor = self.search_matches[idx];
    }

    pub fn prev_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        let idx = match self.search_match_idx {
            Some(0) | None => self.search_matches.len() - 1,
            Some(i) => i - 1,
        };
        self.search_match_idx = Some(idx);
        self.cursor = self.search_matches[idx];
    }

    pub fn show_entities(&mut self) {
        self.overlay = Some(Overlay::Entities);
    }

    pub fn show_attributes(&mut self) {
        if let Some(row) = self.rows.get(self.cursor) {
            self.overlay = Some(Overlay::Attributes(row.element_name.clone()));
        }
    }

    pub fn close_overlay(&mut self) {
        self.overlay = None;
    }
}

fn find_roots(dtd: &Dtd) -> Vec<String> {
    let mut referenced: HashSet<&str> = HashSet::new();
    for elem in dtd.elements.values() {
        collect_refs(&elem.content, &mut referenced);
    }
    dtd.elements
        .keys()
        .filter(|name| !referenced.contains(name.as_str()))
        .cloned()
        .collect()
}

fn collect_refs<'a>(content: &'a ContentModel, names: &mut HashSet<&'a str>) {
    match content {
        ContentModel::Children(group) => collect_group_refs(group, names),
        ContentModel::Mixed(children) => {
            for name in children {
                names.insert(name);
            }
        }
        _ => {}
    }
}

fn collect_group_refs<'a>(group: &'a Group, names: &mut HashSet<&'a str>) {
    for item in &group.items {
        match &item.content {
            GroupItemContent::Name(name) => {
                names.insert(name);
            }
            GroupItemContent::Group(g) => collect_group_refs(g, names),
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
            GroupItemContent::Name(name) => result.push((name.clone(), item.quantifier)),
            GroupItemContent::Group(g) => result.extend(get_group_children(g)),
        }
    }
    result
}
