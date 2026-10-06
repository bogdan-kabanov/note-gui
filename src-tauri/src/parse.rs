use std::path::Path;

use regex::Regex;
use serde_yaml::{Mapping, Value};
use std::sync::LazyLock;

use crate::models::Heading;

static WIKILINK_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[([^\[\]\n]+?)\]\]").expect("wikilink pattern"));

#[derive(Debug, Clone)]
pub struct ParsedLink {
    pub target_name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedNote {
    pub title: String,
    pub property_title: String,
    pub property_tags: Vec<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub headings: Vec<Heading>,
    pub links: Vec<ParsedLink>,
    pub word_count: u32,
}

pub fn parse_note(path: &str, body: &str) -> ParsedNote {
    let (frontmatter, skip_lines, rest) = split_frontmatter(body);
    let mut property_title = String::new();
    let mut property_tags = Vec::new();
    let mut tags = Vec::new();
    let mut aliases = Vec::new();
    if let Some(value) = frontmatter {
        if let Some(title) = value.get("title").and_then(yaml_scalar) {
            property_title = title;
        }
        if let Some(tag_value) = value.get("tags").or_else(|| value.get("tag")) {
            property_tags.extend(yaml_strings(tag_value));
        }
        if let Some(alias_value) = value.get("aliases").or_else(|| value.get("alias")) {
            aliases.extend(yaml_strings(alias_value));
        }
    }
    tags.extend(property_tags.clone());

    let mut headings = Vec::new();
    let mut links = Vec::new();
    let mut in_fence = false;
    let mut fence_mark = String::new();
    for (index, raw_line) in body.lines().enumerate() {
        let line = raw_line.trim_end_matches('\r');
        if index < skip_lines {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let mark = if trimmed.starts_with("```") {
                "```"
            } else {
                "~~~"
            };
            if in_fence && trimmed.starts_with(&fence_mark) {
                in_fence = false;
                fence_mark.clear();
            } else if !in_fence {
                in_fence = true;
                fence_mark = mark.into();
            }
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some((level, text)) = heading_from_line(trimmed) {
            headings.push(Heading {
                level,
                text,
                line: (index as u32) + 1,
            });
        }
        for capture in WIKILINK_PATTERN.captures_iter(line) {
            if let Some(link) = parse_wikilink(&capture[1]) {
                if !links.iter().any(|item: &ParsedLink| {
                    item.target_name == link.target_name && item.alias == link.alias
                }) {
                    links.push(link);
                }
            }
        }
        collect_tags(line, &mut tags);
    }

    dedupe(&mut property_tags);
    dedupe(&mut tags);
    dedupe(&mut aliases);
    let title = effective_title(path, &property_title, &headings);
    let word_count = rest.split_whitespace().count() as u32;
    ParsedNote {
        title,
        property_title: property_title.trim().to_string(),
        property_tags,
        tags,
        aliases,
        headings,
        links,
        word_count,
    }
}

pub fn apply_properties(body: &str, title: &str, tags: &[String], aliases: &[String]) -> String {
    let (frontmatter, _skip_lines, rest) = split_frontmatter(body);
    let mut mapping = match frontmatter {
        Some(Value::Mapping(mapping)) => mapping,
        _ => Mapping::new(),
    };
    set_or_remove(&mut mapping, "title", title);
    set_list_or_remove(&mut mapping, "tags", tags);
    set_list_or_remove(&mut mapping, "aliases", aliases);
    let rest = rest.trim_start_matches(['\n', '\r']);
    if mapping.is_empty() {
        return rest.to_string();
    }
    let yaml = serde_yaml::to_string(&Value::Mapping(mapping)).unwrap_or_default();
    format!("---\n{yaml}---\n{rest}")
}

fn effective_title(path: &str, property_title: &str, headings: &[Heading]) -> String {
    let property_title = property_title.trim();
    if !property_title.is_empty() {
        return property_title.to_string();
    }
    if let Some(heading) = headings.iter().find(|item| item.level == 1) {
        return heading.text.clone();
    }
    if let Some(heading) = headings.first() {
        return heading.text.clone();
    }
    Path::new(path)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn split_frontmatter(text: &str) -> (Option<Value>, usize, String) {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines = text.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return (None, 0, String::new());
    };
    if first.trim() != "---" {
        return (None, 0, text.to_string());
    }
    let mut yaml = String::new();
    let mut rest = String::new();
    let mut closed = false;
    let mut yaml_lines = 0usize;
    for line in lines {
        if !closed && line.trim() == "---" {
            closed = true;
            continue;
        }
        if closed {
            rest.push_str(line);
        } else {
            yaml.push_str(line);
            yaml_lines += 1;
        }
    }
    if !closed {
        return (None, 0, text.to_string());
    }
    match serde_yaml::from_str::<Value>(&yaml) {
        Ok(value) => (Some(value), yaml_lines + 2, rest),
        Err(_) => (None, 0, text.to_string()),
    }
}

fn heading_from_line(line: &str) -> Option<(u8, String)> {
    if !line.starts_with('#') {
        return None;
    }
    let mut level = 0u8;
    for character in line.chars() {
        if character == '#' {
            level = level.saturating_add(1);
        } else {
            break;
        }
    }
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = line.get(level as usize..)?.trim();
    if rest.is_empty() {
        return None;
    }
    let marker_len = level as usize;
    let after_marker = line.as_bytes().get(marker_len).copied()?;
    if after_marker != b' ' && after_marker != b'\t' {
        return None;
    }
    let text = rest.trim_end_matches('#').trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some((level, text))
    }
}

fn parse_wikilink(inner: &str) -> Option<ParsedLink> {
    let (target, alias) = match inner.split_once('|') {
        Some((target, alias)) => (target.trim(), Some(alias.trim().to_string())),
        None => (inner.trim(), None),
    };
    let target = target.split_once('#').map(|(name, _)| name).unwrap_or(target);
    let target = target.split_once('^').map(|(name, _)| name).unwrap_or(target);
    let target_name = target.trim().trim_end_matches(".md").trim().to_string();
    if target_name.is_empty() || target_name.contains("..") {
        return None;
    }
    let alias = alias.filter(|value| !value.is_empty());
    Some(ParsedLink { target_name, alias })
}

fn collect_tags(line: &str, tags: &mut Vec<String>) {
    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '#' {
            let previous_ok = index == 0 || chars[index - 1].is_whitespace();
            let next = chars.get(index + 1).copied();
            if previous_ok && next.is_some_and(|character| character != '#' && is_tag_char(character))
            {
                let start = index + 1;
                let mut end = start;
                while end < chars.len() && is_tag_char(chars[end]) {
                    end += 1;
                }
                let tag: String = chars[start..end].iter().collect();
                if !tag.is_empty() {
                    tags.push(tag);
                }
                index = end;
                continue;
            }
        }
        index += 1;
    }
}

fn is_tag_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_' || character == '-' || character == '/'
}

fn yaml_scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.trim().to_string()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn yaml_strings(value: &Value) -> Vec<String> {
    match value {
        Value::Sequence(items) => items.iter().filter_map(yaml_scalar).collect(),
        Value::String(text) => text
            .split([',', ' '])
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect(),
        other => yaml_scalar(other).into_iter().collect(),
    }
}

fn dedupe(values: &mut Vec<String>) {
    let mut seen = Vec::new();
    values.retain(|value| {
        let key = value.trim().to_string();
        if key.is_empty() || seen.iter().any(|item: &String| item == &key) {
            return false;
        }
        seen.push(key);
        true
    });
    *values = seen;
}

fn set_or_remove(mapping: &mut Mapping, key: &str, value: &str) {
    let key = Value::String(key.into());
    let value = value.trim();
    if value.is_empty() {
        mapping.remove(&key);
    } else {
        mapping.insert(key, Value::String(value.into()));
    }
}

fn set_list_or_remove(mapping: &mut Mapping, key: &str, values: &[String]) {
    let key = Value::String(key.into());
    let items: Vec<Value> = values
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| Value::String(item.to_string()))
        .collect();
    if items.is_empty() {
        mapping.remove(&key);
    } else {
        mapping.insert(key, Value::Sequence(items));
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_properties, parse_note};

    #[test]
    fn parses_links_tags_and_frontmatter() {
        let body = "---\ntitle: Альфа\ntags: [проект]\naliases:\n  - Первая\n---\n# Заголовок\n\nСмотри [[Бета|вторая]] #ещё\n```\n# не тег\n[[скрыто]]\n```\n";
        let parsed = parse_note("папка/Альфа.md", body);
        assert_eq!(parsed.title, "Альфа");
        assert_eq!(parsed.tags, vec!["проект".to_string(), "ещё".to_string()]);
        assert_eq!(parsed.aliases, vec!["Первая".to_string()]);
        assert_eq!(parsed.links.len(), 1);
        assert_eq!(parsed.links[0].target_name, "Бета");
        assert_eq!(parsed.links[0].alias.as_deref(), Some("вторая"));
        assert!(parsed.headings.iter().any(|item| item.text == "Заголовок"));
        assert!(!parsed.links.iter().any(|item| item.target_name == "скрыто"));
    }

    #[test]
    fn rewrites_properties_and_keeps_body() {
        let body = "---\ntitle: Старое\nauthor: я\n---\nТекст\n";
        let next = apply_properties(body, "Новое", &["тег".into()], &[]);
        let parsed = parse_note("Новое.md", &next);
        assert_eq!(parsed.property_title, "Новое");
        assert_eq!(parsed.tags, vec!["тег".to_string()]);
        assert!(next.contains("author"));
        assert!(next.contains("Текст"));
    }
}
