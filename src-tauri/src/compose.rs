use std::collections::HashMap;

use fastrand::Rng;
use serde::Serialize;

use crate::catalog::{Catalog, Category, CategoryEntry, CategoryRange, CatalogRanges, SubjectRange};
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

/// Sample a valid query within the archive's derived ranges and compose it.
///
/// Subject row and each module token are drawn from [`Catalog::ranges`] for the
/// loaded sheet. Sparse sheets may not fill the full level×index rectangle, so
/// picks are constrained to entries that actually exist.
pub fn random_compose(catalog: &Catalog) -> Result<ComposeResult, ComposeError> {
    random_compose_with_rng(catalog, &mut Rng::new())
}

pub(crate) fn random_compose_with_rng(
    catalog: &Catalog,
    rng: &mut Rng,
) -> Result<ComposeResult, ComposeError> {
    let parsed = sample_random_query(catalog, rng)?;
    compose_parsed(catalog, &parsed)
}

fn sample_random_query(catalog: &Catalog, rng: &mut Rng) -> Result<ParsedQuery, ComposeError> {
    let ranges = catalog.ranges();
    let subject_row = pick_subject_row(catalog, &ranges.subjects, rng)?;
    let mut modules = Vec::with_capacity(4);

    for category in [Category::Outfit, Category::Pose, Category::Action] {
        let range = category_range_or_empty(category, &ranges)?;
        modules.push(pick_module_in_range(catalog, category, &range, rng)?);
    }

    if let Some(range) = ranges.scenes.as_ref() {
        modules.push(pick_module_in_range(catalog, Category::Scene, range, rng)?);
    }

    Ok(ParsedQuery {
        subject_row,
        modules,
    })
}

fn category_range_or_empty(
    category: Category,
    ranges: &CatalogRanges,
) -> Result<CategoryRange, ComposeError> {
    let range = match category {
        Category::Outfit => ranges.outfits.as_ref(),
        Category::Pose => ranges.poses.as_ref(),
        Category::Action => ranges.actions.as_ref(),
        Category::Scene => ranges.scenes.as_ref(),
    };
    range.cloned().ok_or_else(|| {
        ComposeError::invalid(
            "empty_catalog",
            format!(
                "{} sheet has no entries — cannot randomize",
                category.sheet_name()
            ),
        )
    })
}

fn pick_subject_row(
    catalog: &Catalog,
    range: &SubjectRange,
    rng: &mut Rng,
) -> Result<u32, ComposeError> {
    let rows: Vec<u32> = catalog
        .subjects
        .keys()
        .copied()
        .filter(|row| *row >= range.min_row && *row <= range.max_row)
        .collect();
    if rows.is_empty() {
        return Err(ComposeError::invalid(
            "empty_catalog",
            "Subjects sheet has no entries — cannot randomize",
        ));
    }

    // Prefer uniform draws over the published subject row range, then fall back
    // to an existing row when the rectangle has gaps.
    for _ in 0..64 {
        let row = rng.u32(range.min_row..=range.max_row);
        if catalog.subjects.contains_key(&row) {
            return Ok(row);
        }
    }
    Ok(rows[rng.usize(0..rows.len())])
}

fn pick_module_in_range(
    catalog: &Catalog,
    category: Category,
    range: &CategoryRange,
    rng: &mut Rng,
) -> Result<ModuleToken, ComposeError> {
    let map: &HashMap<(u8, u8), CategoryEntry> = catalog.category_map(category);
    let keys: Vec<(u8, u8)> = map
        .keys()
        .copied()
        .filter(|&(level, index)| in_category_range(level, index, range))
        .collect();
    if keys.is_empty() {
        return Err(ComposeError::invalid(
            "empty_catalog",
            format!(
                "{} sheet has no entries in archive range L{}–{} / I{}–{} — cannot randomize",
                category.sheet_name(),
                range.min_level,
                range.max_level,
                range.min_index,
                range.max_index
            ),
        ));
    }

    // Draw level/index inside the sheet's published range; accept only real entries.
    for _ in 0..64 {
        let level = rng.u8(range.min_level..=range.max_level);
        let index = rng.u8(range.min_index..=range.max_index);
        if map.contains_key(&(level, index)) {
            return Ok(ModuleToken {
                level,
                index,
                category,
            });
        }
    }

    let (level, index) = keys[rng.usize(0..keys.len())];
    Ok(ModuleToken {
        level,
        index,
        category,
    })
}

fn in_category_range(level: u8, index: u8, range: &CategoryRange) -> bool {
    level >= range.min_level
        && level <= range.max_level
        && index >= range.min_index
        && index <= range.max_index
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

    #[test]
    fn random_compose_succeeds_and_is_recomposible() {
        let catalog = load_fixture();
        assert!(!catalog.scenes.is_empty(), "fixture should have scenes");
        let ranges = catalog.ranges();

        let mut rng = Rng::with_seed(42);
        for _ in 0..20 {
            let result = random_compose_with_rng(&catalog, &mut rng).unwrap();
            assert_eq!(result.parts.len(), 5, "subject + O/P/A + scene");
            assert_eq!(result.parts[0].kind, "subject");
            assert_eq!(result.parts[1].kind, "outfit");
            assert_eq!(result.parts[2].kind, "pose");
            assert_eq!(result.parts[3].kind, "action");
            assert_eq!(result.parts[4].kind, "scene");

            let parsed = parse_query(&result.query).unwrap();
            assert!(
                parsed.subject_row >= ranges.subjects.min_row
                    && parsed.subject_row <= ranges.subjects.max_row
            );
            assert_module_in_range(&parsed.modules[0], ranges.outfits.as_ref().unwrap());
            assert_module_in_range(&parsed.modules[1], ranges.poses.as_ref().unwrap());
            assert_module_in_range(&parsed.modules[2], ranges.actions.as_ref().unwrap());
            assert_module_in_range(&parsed.modules[3], ranges.scenes.as_ref().unwrap());

            let again = compose_from_query(&catalog, &result.query).unwrap();
            assert_eq!(again.prompt, result.prompt);
            assert_eq!(again.query, result.query);
            assert_eq!(again.parts, result.parts);
        }
    }

    fn assert_module_in_range(module: &ModuleToken, range: &CategoryRange) {
        assert!(
            module.level >= range.min_level && module.level <= range.max_level,
            "{} level {} outside {}–{}",
            module.category.label(),
            module.level,
            range.min_level,
            range.max_level
        );
        assert!(
            module.index >= range.min_index && module.index <= range.max_index,
            "{} index {} outside {}–{}",
            module.category.label(),
            module.index,
            range.min_index,
            range.max_index
        );
    }

    #[test]
    fn random_compose_omits_scene_when_empty() {
        let mut catalog = load_fixture();
        catalog.scenes.clear();

        let mut rng = Rng::with_seed(7);
        let result = random_compose_with_rng(&catalog, &mut rng).unwrap();
        assert_eq!(result.parts.len(), 4);
        assert_eq!(result.parts[3].kind, "action");
        let tokens: Vec<_> = result.query.split_whitespace().collect();
        assert_eq!(tokens.len(), 4); // row + 3 modules
    }

    #[test]
    fn random_compose_errors_when_required_category_empty() {
        let mut catalog = load_fixture();
        catalog.poses.clear();
        let err = random_compose(&catalog).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Poses") || format!("{err:?}").contains("empty_catalog"),
            "unexpected error: {msg}"
        );
    }
}
