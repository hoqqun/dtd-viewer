use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::model::*;
use super::state::{AppState, Overlay};

pub fn render(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(f.area());

    render_tree(f, chunks[0], state);
    render_help_bar(f, chunks[1], state);

    if let Some(ref overlay) = state.overlay {
        render_overlay(f, overlay, state);
    }
}

fn render_tree(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(" DTD Viewer ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = state
        .rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let indent = "  ".repeat(row.depth);
            let icon = if !row.has_children {
                "  "
            } else if row.is_expanded {
                "▼ "
            } else {
                "▶ "
            };

            let q = format!("{}", row.quantifier);

            let attrs = state
                .dtd
                .elements
                .get(&row.element_name)
                .map(|e| format_inline_attrs(&e.attributes))
                .unwrap_or_default();

            let child_count = if !row.is_expanded && row.has_children {
                let count = state
                    .dtd
                    .elements
                    .get(&row.element_name)
                    .map(|e| count_children(&e.content))
                    .unwrap_or(0);
                if count > 0 {
                    format!(" ({} children)", count)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let text = format!("{}{}{}{}{}{}", indent, icon, row.element_name, q, attrs, child_count);

            let style = if i == state.cursor {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if state.search_matches.contains(&i) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(text).style(style)
        })
        .collect();

    // Calculate offset for scrolling
    let visible_height = inner.height as usize;
    let offset = if state.cursor >= visible_height {
        state.cursor - visible_height + 1
    } else {
        0
    };

    // Render with manual offset
    let _items = items; // consumed above for building
    let visible_items: Vec<ListItem> = state
        .rows
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_height)
        .map(|(i, row)| {
            let indent = "  ".repeat(row.depth);
            let icon = if !row.has_children {
                "  "
            } else if row.is_expanded {
                "▼ "
            } else {
                "▶ "
            };
            let q = format!("{}", row.quantifier);
            let attrs = state
                .dtd
                .elements
                .get(&row.element_name)
                .map(|e| format_inline_attrs(&e.attributes))
                .unwrap_or_default();
            let child_count = if !row.is_expanded && row.has_children {
                let count = state
                    .dtd
                    .elements
                    .get(&row.element_name)
                    .map(|e| count_children(&e.content))
                    .unwrap_or(0);
                if count > 0 {
                    format!(" ({} children)", count)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            let text = format!("{}{}{}{}{}{}", indent, icon, row.element_name, q, attrs, child_count);
            let style = if i == state.cursor {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if state.search_matches.contains(&i) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let visible_list = List::new(visible_items);
    f.render_widget(visible_list, inner);
}

fn render_help_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let text = if state.search_mode {
        format!("Search: {}█", state.search_query)
    } else {
        "[↑↓] move  [Enter/→] expand  [←] collapse  [/] search  [e] entities  [a] attributes  [q] quit".to_string()
    };
    let block = Block::default().borders(Borders::TOP);
    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

fn render_overlay(f: &mut Frame, overlay: &Overlay, state: &AppState) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    match overlay {
        Overlay::Entities => {
            let mut lines = Vec::new();
            if state.dtd.entities.is_empty() {
                lines.push(Line::from("No entities defined."));
            } else {
                for entity in &state.dtd.entities {
                    let prefix = if entity.is_parameter { "%" } else { "&" };
                    let desc = match &entity.kind {
                        EntityKind::Internal { value } => format!("\"{}\"", value),
                        EntityKind::ExternalSystem { uri } => format!("SYSTEM \"{}\"", uri),
                        EntityKind::ExternalPublic { public_id, uri } => {
                            format!("PUBLIC \"{}\" \"{}\"", public_id, uri)
                        }
                    };
                    lines.push(Line::from(format!("  {}{}; = {}", prefix, entity.name, desc)));
                }
            }
            let block = Block::default()
                .title(" Entities ")
                .borders(Borders::ALL);
            let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
            f.render_widget(para, area);
        }
        Overlay::Attributes(elem_name) => {
            let mut lines = Vec::new();
            if let Some(elem) = state.dtd.elements.get(elem_name) {
                if elem.attributes.is_empty() {
                    lines.push(Line::from("No attributes defined."));
                } else {
                    for attr in &elem.attributes {
                        lines.push(Line::from(format!(
                            "  @{}: {} {}",
                            attr.name, attr.attr_type, attr.default
                        )));
                    }
                }
            }
            let block = Block::default()
                .title(format!(" Attributes: {} ", elem_name))
                .borders(Borders::ALL);
            let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
            f.render_widget(para, area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn format_inline_attrs(attrs: &[Attribute]) -> String {
    if attrs.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = attrs
        .iter()
        .map(|a| format!("@{}: {}", a.name, a.attr_type))
        .collect();
    format!(" [{}]", parts.join(", "))
}

fn count_children(content: &ContentModel) -> usize {
    match content {
        ContentModel::Children(group) => count_group_children(group),
        ContentModel::Mixed(names) => names.len(),
        _ => 0,
    }
}

fn count_group_children(group: &Group) -> usize {
    let mut count = 0;
    for item in &group.items {
        match &item.content {
            GroupItemContent::Name(_) => count += 1,
            GroupItemContent::Group(g) => count += count_group_children(g),
        }
    }
    count
}
