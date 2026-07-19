use crate::catalog::{Category, MAX_INDEX, MAX_OPERATIONAL_LEVEL};
use crate::error::ComposeError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleToken {
    pub level: u8,
    pub index: u8,
    pub category: Category,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQuery {
    pub subject_row: u32,
    pub modules: Vec<ModuleToken>,
}

pub fn parse_query(input: &str) -> Result<ParsedQuery, ComposeError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ComposeError::invalid(
            "empty",
            "query is empty — expected e.g. `7 1lvl5 2lvl3 1lvl4`",
        ));
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(ComposeError::invalid(
            "incomplete",
            "expected `<row>` followed by 1–4 `NlvlM` tokens",
        ));
    }

    let first = tokens[0];
    if !first.chars().all(|c| c.is_ascii_digit()) {
        return Err(ComposeError::invalid(
            "unknown_keyword",
            format!("expected a subject row number, got `{first}`"),
        ));
    }

    let row_token = tokens[0];
    let module_tokens = &tokens[1..];

    let subject_row: u32 = row_token.parse().map_err(|_| {
        ComposeError::invalid(
            "subject_row_invalid",
            format!("subject row must be an integer, got `{row_token}`"),
        )
    })?;
    if subject_row < 2 {
        return Err(ComposeError::invalid(
            "subject_row_invalid",
            format!("subject row must be >= 2 (Excel data rows), got {subject_row}"),
        ));
    }

    if module_tokens.is_empty() {
        return Err(ComposeError::invalid(
            "missing_modules",
            "add at least one module token like `1lvl5` (Outfit), then Pose/Action/Scene",
        ));
    }
    if module_tokens.len() > 4 {
        return Err(ComposeError::invalid(
            "too_many_modules",
            format!(
                "expected at most 4 module tokens (Outfit Pose Action Scene), got {}",
                module_tokens.len()
            ),
        ));
    }

    let mut modules = Vec::with_capacity(module_tokens.len());
    for (slot, raw) in module_tokens.iter().enumerate() {
        let (level, index) = parse_module_token(raw)?;
        let category = Category::from_slot(slot).expect("slot checked by len");
        modules.push(ModuleToken {
            level,
            index,
            category,
        });
    }

    Ok(ParsedQuery {
        subject_row,
        modules,
    })
}

fn parse_module_token(raw: &str) -> Result<(u8, u8), ComposeError> {
    let lower = raw.to_ascii_lowercase();

    // Canonical: NlvlM  e.g. 1lvl5
    if let Some((level_s, index_s)) = lower.split_once("lvl") {
        if !level_s.is_empty()
            && !index_s.is_empty()
            && level_s.chars().all(|c| c.is_ascii_digit())
            && index_s.chars().all(|c| c.is_ascii_digit())
        {
            let level: u8 = level_s.parse().map_err(|_| {
                ComposeError::invalid("level_invalid", format!("invalid level in `{raw}`"))
            })?;
            let index: u8 = index_s.parse().map_err(|_| {
                ComposeError::invalid("index_invalid", format!("invalid index in `{raw}`"))
            })?;
            validate_level_index(level, index, raw)?;
            return Ok((level, index));
        }
    }

    // Friendly suggestions for common typos
    let suggestion = if lower.contains("level") {
        Some("use NlvlM (e.g. 1lvl5), not `level`".to_string())
    } else if lower.contains("lv") && !lower.contains("lvl") {
        Some("did you mean NlvlM? (two letters: lvl)".to_string())
    } else if lower.starts_with("lvl") {
        Some("put the level number first, e.g. 1lvl5".to_string())
    } else {
        None
    };

    Err(match suggestion {
        Some(s) => ComposeError::invalid_suggest(
            "malformed_module",
            format!("malformed module `{raw}` — expected pattern NlvlM (e.g. 1lvl5)"),
            s,
        ),
        None => ComposeError::invalid(
            "malformed_module",
            format!("malformed module `{raw}` — expected pattern NlvlM (e.g. 1lvl5)"),
        ),
    })
}

fn validate_level_index(level: u8, index: u8, raw: &str) -> Result<(), ComposeError> {
    if level == 0 || level > 7 {
        return Err(ComposeError::invalid(
            "level_out_of_range",
            format!("level in `{raw}` must be 1–5 (got {level})"),
        ));
    }
    if level > MAX_OPERATIONAL_LEVEL {
        return Err(ComposeError::invalid(
            "level_blocked",
            format!(
                "level {level} in `{raw}` is red/black-listed — operational maximum is level {MAX_OPERATIONAL_LEVEL}"
            ),
        ));
    }
    if index == 0 || index > MAX_INDEX {
        return Err(ComposeError::invalid(
            "index_out_of_range",
            format!("index in `{raw}` must be 1–{MAX_INDEX} (got {index})"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures_data::GOLDEN_QUERY;

    #[test]
    fn parses_example() {
        let q = parse_query("7 1lvl5 2lvl3 1lvl4").unwrap();
        assert_eq!(q.subject_row, 7);
        assert_eq!(q.modules.len(), 3);
        assert_eq!(q.modules[0].level, 1);
        assert_eq!(q.modules[0].index, 5);
        assert_eq!(q.modules[0].category, Category::Outfit);
        assert_eq!(q.modules[1].level, 2);
        assert_eq!(q.modules[1].index, 3);
        assert_eq!(q.modules[1].category, Category::Pose);
        assert_eq!(q.modules[2].level, 1);
        assert_eq!(q.modules[2].index, 4);
        assert_eq!(q.modules[2].category, Category::Action);
    }

    #[test]
    fn parses_golden_fixture_query() {
        let q = parse_query(GOLDEN_QUERY).unwrap();
        assert_eq!(q.subject_row, 2);
        assert_eq!(q.modules.len(), 3);
        assert_eq!(q.modules[0].category, Category::Outfit);
        assert_eq!(q.modules[1].category, Category::Pose);
        assert_eq!(q.modules[2].category, Category::Action);
    }

    #[test]
    fn rejects_bare_row_without_modules() {
        assert!(parse_query("2").is_err());
    }

    #[test]
    fn rejects_non_numeric_prefix() {
        let err = parse_query("abc 7 1lvl5").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("row") || format!("{err:?}").contains("unknown_keyword"));
    }

    #[test]
    fn rejects_level_6() {
        let err = parse_query("7 6lvl1").unwrap_err();
        assert!(err.to_string().contains("red/black") || err.to_string().contains("maximum"));
    }

    #[test]
    fn rejects_malformed() {
        assert!(parse_query("7 1lv5").is_err());
        assert!(parse_query("7 lvl5").is_err());
    }

    #[test]
    fn rejects_empty_and_too_many() {
        assert!(parse_query("").is_err());
        assert!(parse_query("2").is_err());
        assert!(parse_query("2 1lvl1 2lvl1 1lvl2 3lvl1 1lvl1").is_err());
    }
}
