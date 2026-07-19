use serde::Serialize;

use crate::catalog::Catalog;
use crate::error::ComposeError;
use crate::parse::{parse_query, ParsedQuery};

#[derive(Debug, Clone, Serialize)]
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
        self, fixture_path, GOLDEN_PROMPT, GOLDEN_PROMPT_WITH_SCENE, GOLDEN_QUERY,
        GOLDEN_QUERY_WITH_SCENE,
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
