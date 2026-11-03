use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_yaml::Value;
use tauri::State;

use crate::workspace::{WorkspaceState, is_skill_file, walk_markdown};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    /// Workspace-relative path.
    pub path: String,
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub title: Option<String>,
    pub valid: bool,
    pub issues: Vec<String>,
}

/// Split a Markdown document into `(frontmatter yaml, body)`.
/// Returns `(None, original)` when there is no leading `---` block.
pub fn split_frontmatter(input: &str) -> (Option<&str>, &str) {
    let text = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut lines = text.split_inclusive('\n');

    let Some(first) = lines.next() else {
        return (None, input);
    };
    if first.trim_end_matches(['\r', '\n']).trim() != "---" {
        return (None, input);
    }

    let mut offset = first.len();
    for line in lines {
        let clean = line.trim_end_matches(['\r', '\n']).trim();
        offset += line.len();
        if clean == "---" || clean == "..." {
            let yaml = &text[first.len()..offset - line.len()];
            return (Some(yaml), &text[offset..]);
        }
    }
    (None, input)
}

fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        }
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn push_triggers(haystack: &Value, out: &mut Vec<String>) {
    match haystack {
        Value::Sequence(items) => {
            for item in items {
                if let Some(text) = text_of(item) {
                    for part in text.split(',') {
                        let part = part.trim();
                        if !part.is_empty() && !out.iter().any(|t| t.eq_ignore_ascii_case(part)) {
                            out.push(part.to_string());
                        }
                    }
                }
            }
        }
        Value::String(text) => {
            for part in text.split(',') {
                let part = part.trim();
                if !part.is_empty() && !out.iter().any(|t| t.eq_ignore_ascii_case(part)) {
                    out.push(part.to_string());
                }
            }
        }
        _ => {}
    }
}

fn first_heading(body: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.trim().strip_prefix("# ").map(|h| h.trim().to_string()))
        .filter(|h| !h.is_empty())
}

/// Parse a skill document. `folder` is the containing directory name, used as a fallback
/// name and to check the `name:` field against the on-disk convention.
pub fn parse_skill(rel_path: &str, content: &str) -> SkillInfo {
    let folder = Path::new(rel_path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("skill")
        .to_string();

    let (yaml, body) = split_frontmatter(content);
    let title = first_heading(body);
    let mut issues = Vec::new();

    let Some(yaml) = yaml else {
        return SkillInfo {
            path: rel_path.to_string(),
            name: folder,
            description: String::new(),
            triggers: Vec::new(),
            title,
            valid: false,
            issues: vec!["Missing YAML frontmatter".to_string()],
        };
    };

    let parsed: Result<Value, _> = serde_yaml::from_str(yaml);
    let value = match parsed {
        Ok(value) => value,
        Err(error) => {
            return SkillInfo {
                path: rel_path.to_string(),
                name: folder,
                description: String::new(),
                triggers: Vec::new(),
                title,
                valid: false,
                issues: vec![format!("Invalid YAML: {error}")],
            };
        }
    };

    let Some(map) = value.as_mapping() else {
        return SkillInfo {
            path: rel_path.to_string(),
            name: folder,
            description: String::new(),
            triggers: Vec::new(),
            title,
            valid: false,
            issues: vec!["Frontmatter must be a YAML mapping".to_string()],
        };
    };

    let field = |key: &str| map.get(&Value::String(key.to_string()));

    let name = field("name").and_then(text_of).unwrap_or_else(|| {
        issues.push("Missing `name`".to_string());
        folder.clone()
    });
    let description = field("description").and_then(text_of).unwrap_or_else(|| {
        issues.push("Missing `description`".to_string());
        String::new()
    });

    let mut triggers = Vec::new();
    for key in ["triggers", "trigger", "when_to_use", "whenToUse"] {
        if let Some(value) = field(key) {
            push_triggers(value, &mut triggers);
        }
    }

    if name != folder {
        issues.push(format!("`name` does not match folder `{folder}`"));
    }

    SkillInfo {
        path: rel_path.to_string(),
        name,
        description,
        triggers,
        title,
        valid: issues.is_empty(),
        issues,
    }
}

#[tauri::command]
pub fn scan_skills(state: State<'_, WorkspaceState>) -> Result<Vec<SkillInfo>, String> {
    let root = state
        .root
        .lock()
        .map_err(|_| "workspace state poisoned".to_string())?
        .clone()
        .ok_or_else(|| "no workspace is open".to_string())?;

    let mut skills = Vec::new();
    for path in walk_markdown(&root) {
        if !is_skill_file(&path) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        skills.push(parse_skill(&rel, &content));
    }

    skills.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });
    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;
    const GOOD: &str = "---\nname: code-review\ndescription: Review code changes\ntriggers:\n  - review\n  - code review\n---\n\n# Code Review\n\nBody.\n";

    #[test]
    fn parses_frontmatter_and_body() {
        let (yaml, body) = split_frontmatter(GOOD);
        assert!(yaml.unwrap().contains("name: code-review"));
        assert!(body.trim_start().starts_with("# Code Review"));
    }

    #[test]
    fn good_skill_is_valid_and_has_triggers() {
        let skill = parse_skill("skills/code-review/SKILL.md", GOOD);
        assert!(skill.valid, "{:?}", skill.issues);
        assert_eq!(skill.name, "code-review");
        assert_eq!(skill.description, "Review code changes");
        assert_eq!(skill.triggers, vec!["review", "code review"]);
        assert_eq!(skill.title.as_deref(), Some("Code Review"));
    }

    #[test]
    fn missing_frontmatter_is_reported() {
        let skill = parse_skill("skills/foo/SKILL.md", "# Foo\n");
        assert!(!skill.valid);
        assert_eq!(skill.issues[0], "Missing YAML frontmatter");
        assert_eq!(skill.name, "foo");
    }

    #[test]
    fn invalid_yaml_is_reported_not_panicked() {
        let skill = parse_skill("skills/foo/SKILL.md", "---\nname: [unclosed\n---\nbody");
        assert!(!skill.valid);
        assert!(skill.issues[0].starts_with("Invalid YAML"));
    }

    #[test]
    fn name_mismatch_with_folder_is_flagged() {
        let skill = parse_skill(
            "skills/other/SKILL.md",
            "---\nname: code-review\ndescription: x\n---\n",
        );
        assert!(!skill.valid);
        assert!(
            skill
                .issues
                .iter()
                .any(|i| i.contains("does not match folder"))
        );
    }

    #[test]
    fn missing_fields_are_flagged() {
        let skill = parse_skill("skills/foo/SKILL.md", "---\nname: foo\n---\n");
        assert!(!skill.valid);
        assert!(skill.issues.iter().any(|i| i.contains("description")));
    }

    #[test]
    fn string_triggers_split_on_commas() {
        let skill = parse_skill(
            "skills/foo/SKILL.md",
            "---\nname: foo\ndescription: d\ntriggers: a, b\n---\n",
        );
        assert_eq!(skill.triggers, vec!["a", "b"]);
        assert!(skill.valid, "{:?}", skill.issues);
    }

    #[test]
    fn crlf_frontmatter_still_parses() {
        let crlf = GOOD.replace('\n', "\r\n");
        let skill = parse_skill("skills/code-review/SKILL.md", &crlf);
        assert!(skill.valid, "{:?}", skill.issues);
    }

    #[test]
    fn bom_is_ignored() {
        let with_bom = format!("\u{feff}{GOOD}");
        let skill = parse_skill("skills/code-review/SKILL.md", &with_bom);
        assert!(skill.valid, "{:?}", skill.issues);
    }
}
