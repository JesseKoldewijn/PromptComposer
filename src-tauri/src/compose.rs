use serde::Serialize;

use crate::catalog::{Catalog, CategoryRange};
use crate::error::ComposeError;
use crate::parse::{parse_query, ModuleToken, ParsedQuery};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromptPart {
    pub kind: String,
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComposeResult {
    pub prompt: String,
    pub parts: Vec<PromptPart>,
    pub query: String,
}

pub fn compose_from_query(catalog: &Catalog, query: &str) -> Result<ComposeResult, ComposeError> {
    let parsed = parse_query(query)?;
    compose_parsed(catalog, &parsed)
}

fn compose_parsed(catalog: &Catalog, parsed: &ParsedQuery) -> Result<ComposeResult, ComposeError> {
    validate_against_ranges(catalog, parsed)?;

    let subject = catalog.subject(parsed.subject_row)?;
    let mut parts = Vec::new();
    parts.push(PromptPart {
        kind: "subject".into(),
        label: format!("Subject row {} ({})", subject.row, subject.name),
        text: subject.body.clone(),
    });

    for module in &parsed.modules {
        let entry = catalog.entry(module.category, module.level, module.index)?;
        parts.push(PromptPart {
            kind: format!("{:?}", module.category).to_ascii_lowercase(),
            label: entry.name.clone(),
            text: entry.prompt.clone(),
        });
    }

    let prompt = parts
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(ComposeResult {
        prompt,
        parts,
        query: format_canonical_query(parsed),
    })
}

fn validate_against_ranges(catalog: &Catalog, parsed: &ParsedQuery) -> Result<(), ComposeError> {
    let ranges = catalog.ranges();
    let subjects = &ranges.subjects;
    if parsed.subject_row < subjects.min_row || parsed.subject_row > subjects.max_row {
        return Err(ComposeError::invalid(
            "subject_out_of_range",
            format!(
                "subject row {} is outside archive range {}–{}",
                parsed.subject_row, subjects.min_row, subjects.max_row
            ),
        ));
    }

    for module in &parsed.modules {
        validate_module_against_range(catalog, module)?;
    }
    Ok(())
}

fn validate_module_against_range(
    catalog: &Catalog,
    module: &ModuleToken,
) -> Result<(), ComposeError> {
    let Some(range) = catalog.category_range_for(module.category) else {
        return Err(ComposeError::invalid(
            "entry_not_found",
            format!(
                "{} sheet has no entries in the loaded archive",
                module.category.sheet_name()
            ),
        ));
    };
    validate_level_index_against_range(module, &range)
}

fn validate_level_index_against_range(
    module: &ModuleToken,
    range: &CategoryRange,
) -> Result<(), ComposeError> {
    let label = module.category.label();
    if module.level < range.min_level || module.level > range.max_level {
        return Err(ComposeError::invalid(
            "level_out_of_range",
            format!(
                "{label} level must be {}–{} (got {})",
                range.min_level, range.max_level, module.level
            ),
        ));
    }
    if module.index < range.min_index || module.index > range.max_index {
        return Err(ComposeError::invalid(
            "index_out_of_range",
            format!(
                "{label} index must be {}–{} (got {})",
                range.min_index, range.max_index, module.index
            ),
        ));
    }
    Ok(())
}

fn format_canonical_query(parsed: &ParsedQuery) -> String {
    let mut out = format!("{}", parsed.subject_row);
    for m in &parsed.modules {
        out.push(' ');
        out.push_str(&format!("{}lvl{}", m.level, m.index));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Category;
    use crate::fixtures_data::{
        self, fixture_path, ALT_PROMPT, ALT_PROMPT_WITH_SCENE, ALT_QUERY, ALT_QUERY_WITH_SCENE,
        GOLDEN_PROMPT, GOLDEN_PROMPT_WITH_SCENE, GOLDEN_QUERY, GOLDEN_QUERY_SLASH,
        GOLDEN_QUERY_WITH_SCENE, OUTFIT_ONLY_PROMPT, OUTFIT_ONLY_QUERY,
    };

    fn load_fixture() -> Catalog {
        Catalog::load(&fixture_path()).expect("load fixture catalog")
    }

    #[test]
    fn golden_example_full_prompt() {
        let catalog = load_fixture();
        let result = compose_from_query(&catalog, GOLDEN_QUERY).unwrap();
        assert_eq!(result.prompt, GOLDEN_PROMPT);
        assert_eq!(result.parts.len(), 4);
        assert_eq!(result.query, GOLDEN_QUERY);
    }

    #[test]
    fn golden_with_scene() {
        let catalog = load_fixture();
        let result = compose_from_query(&catalog, GOLDEN_QUERY_WITH_SCENE).unwrap();
        assert_eq!(result.prompt, GOLDEN_PROMPT_WITH_SCENE);
        assert_eq!(result.parts.len(), 5);
    }

    #[test]
    fn slash_shorthand_composes_same_as_lvl() {
        let catalog = load_fixture();
        let lvl = compose_from_query(&catalog, GOLDEN_QUERY).unwrap();
        let slash = compose_from_query(&catalog, GOLDEN_QUERY_SLASH).unwrap();
        assert_eq!(slash.prompt, lvl.prompt);
        assert_eq!(slash.query, GOLDEN_QUERY);
        assert_eq!(slash.parts, lvl.parts);
    }

    #[test]
    fn alternate_subject_and_levels() {
        let catalog = load_fixture();
        let result = compose_from_query(&catalog, ALT_QUERY).unwrap();
        assert_eq!(result.prompt, ALT_PROMPT);
        assert_eq!(result.parts.len(), 4);
        assert_eq!(result.query, ALT_QUERY);
    }

    #[test]
    fn alternate_with_scene() {
        let catalog = load_fixture();
        let result = compose_from_query(&catalog, ALT_QUERY_WITH_SCENE).unwrap();
        assert_eq!(result.prompt, ALT_PROMPT_WITH_SCENE);
        assert_eq!(result.parts.len(), 5);
    }

    #[test]
    fn outfit_only_query() {
        let catalog = load_fixture();
        let result = compose_from_query(&catalog, OUTFIT_ONLY_QUERY).unwrap();
        assert_eq!(result.prompt, OUTFIT_ONLY_PROMPT);
        assert_eq!(result.parts.len(), 2);
    }

    #[test]
    fn missing_subject_row_errors() {
        let catalog = load_fixture();
        let err = compose_from_query(&catalog, "99 1lvl1").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("outside archive range")
                || msg.contains("not found")
                || format!("{err:?}").contains("subject_out_of_range"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn missing_category_entry_errors() {
        let catalog = load_fixture();
        // Within Outfit ceilings (L1–5 / I1–30) but no Outfit L3-01 in the fixture.
        let err = compose_from_query(&catalog, "2 3lvl1").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("not found") || format!("{err:?}").contains("entry_not_found"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn rejects_level_above_category_max() {
        let catalog = load_fixture();
        // Outfit max level in fixture is 5.
        let err = compose_from_query(&catalog, "2 6lvl1").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("level") && (msg.contains("1–5") || msg.contains("1-5")),
            "unexpected error: {msg}"
        );
        assert!(format!("{err:?}").contains("level_out_of_range") || msg.contains("level"));
    }

    #[test]
    fn rejects_index_above_category_max() {
        let catalog = load_fixture();
        // Outfit max index in fixture is 30.
        let err = compose_from_query(&catalog, "2 1lvl31").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("index") && (msg.contains("1–30") || msg.contains("1-30")),
            "unexpected error: {msg}"
        );
        assert!(format!("{err:?}").contains("index_out_of_range") || msg.contains("index"));
    }

    #[test]
    fn rejects_index_below_category_min() {
        let catalog = load_fixture();
        // Action min index in fixture is 2.
        let err = compose_from_query(&catalog, "2 1lvl1 2lvl1 1lvl1").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Action") && msg.contains("index"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn category_slots_match_token_order() {
        let catalog = load_fixture();
        let result = compose_from_query(&catalog, GOLDEN_QUERY).unwrap();
        assert_eq!(result.parts[0].kind, "subject");
        assert_eq!(result.parts[1].kind, "outfit");
        assert_eq!(result.parts[2].kind, "pose");
        assert_eq!(result.parts[3].kind, "action");
        assert_eq!(
            catalog
                .entry(
                    Category::Outfit,
                    fixtures_data::OUTFIT_LEVEL,
                    fixtures_data::OUTFIT_INDEX
                )
                .unwrap()
                .prompt,
            fixtures_data::OUTFIT_PROMPT
        );
    }
}
