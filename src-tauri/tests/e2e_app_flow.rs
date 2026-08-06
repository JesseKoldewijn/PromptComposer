//! End-to-end flow through the same archive/compose APIs the UI invokes.
//! (GUI WebdriverIO specs live under /e2e; this covers the full backend path.)

use app_lib::archive::{
    clear_archive_in, compose_with_state, import_archive_into, load_state_from_dir,
    random_compose_with_state, AppState,
};
use app_lib::catalog::CategoryRange;
use app_lib::fixtures_data::{fixture_path, ALT_PROMPT, ALT_QUERY, GOLDEN_PROMPT, GOLDEN_QUERY};
use app_lib::parse::{parse_query, ModuleToken};
use tempfile::tempdir;

#[test]
fn e2e_empty_import_compose_error_clear() {
    let dir = tempdir().unwrap();
    let data = dir.path();
    let fixture = fixture_path();
    let mut state = AppState::default();

    // Cold start: no archive
    let err = compose_with_state(&state, GOLDEN_QUERY).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("no archive") || format!("{err:?}").contains("no_archive"),
        "unexpected: {msg}"
    );

    let random_err = random_compose_with_state(&state).unwrap_err();
    assert!(
        random_err.to_string().contains("no archive")
            || format!("{random_err:?}").contains("no_archive"),
        "unexpected: {random_err}"
    );

    // Import fixture (same outcome as import_archive_from_path)
    let status = import_archive_into(&mut state, &fixture, data).unwrap();
    assert!(status.loaded);
    assert_eq!(
        status.original_name.as_deref(),
        Some("minimal_prompt_archive.xlsx")
    );

    // Compose golden + alternate subject/levels
    let result = compose_with_state(&state, GOLDEN_QUERY).unwrap();
    assert_eq!(result.prompt, GOLDEN_PROMPT);
    assert_eq!(result.query, GOLDEN_QUERY);
    assert_eq!(result.parts.len(), 4);

    assert_eq!(
        compose_with_state(&state, ALT_QUERY).unwrap().prompt,
        ALT_PROMPT
    );

    // Missing catalog entry (valid syntax)
    let missing = compose_with_state(&state, "2 3lvl1").unwrap_err();
    assert!(
        missing.to_string().contains("not found")
            || format!("{missing:?}").contains("entry_not_found"),
        "unexpected: {missing}"
    );

    // Bad query validation
    let bad = compose_with_state(&state, "abc 2 1lvl1").unwrap_err();
    assert!(
        bad.to_string().contains("row") || format!("{bad:?}").contains("unknown_keyword"),
        "unexpected: {bad}"
    );

    // Persist + reload (app restart)
    let reloaded = load_state_from_dir(data).unwrap();
    assert!(reloaded.catalog.is_some());
    assert_eq!(
        compose_with_state(&reloaded, GOLDEN_QUERY).unwrap().prompt,
        GOLDEN_PROMPT
    );

    // Clear
    let cleared = clear_archive_in(&mut state, data).unwrap();
    assert!(!cleared.loaded);
    assert!(compose_with_state(&state, GOLDEN_QUERY).is_err());
}

#[test]
fn e2e_random_compose_stays_within_archive_ranges() {
    let dir = tempdir().unwrap();
    let data = dir.path();
    let mut state = AppState::default();

    let status = import_archive_into(&mut state, &fixture_path(), data).unwrap();
    let ranges = status.ranges.expect("fixture ranges");
    let outfits = ranges.outfits.expect("outfit ranges");
    let poses = ranges.poses.expect("pose ranges");
    let actions = ranges.actions.expect("action ranges");
    let scenes = ranges.scenes.expect("scene ranges");

    assert_eq!(ranges.subjects.min_row, 2);
    assert_eq!(ranges.subjects.max_row, 3);
    assert_eq!(outfits.min_level, 1);
    assert_eq!(outfits.max_level, 5);
    assert_eq!(outfits.min_index, 1);
    assert_eq!(outfits.max_index, 30);

    let mut seen = std::collections::HashSet::new();
    for _ in 0..25 {
        let result = random_compose_with_state(&state).unwrap();
        assert_eq!(result.parts.len(), 5, "fixture includes scenes");
        assert!(!result.prompt.trim().is_empty());

        let parsed = parse_query(&result.query).unwrap();
        assert!(
            parsed.subject_row >= ranges.subjects.min_row
                && parsed.subject_row <= ranges.subjects.max_row,
            "subject row {} outside {}–{}",
            parsed.subject_row,
            ranges.subjects.min_row,
            ranges.subjects.max_row
        );
        assert_eq!(parsed.modules.len(), 4);
        assert_module_in_range(&parsed.modules[0], &outfits);
        assert_module_in_range(&parsed.modules[1], &poses);
        assert_module_in_range(&parsed.modules[2], &actions);
        assert_module_in_range(&parsed.modules[3], &scenes);

        // Random query must recompose identically through the normal path.
        let again = compose_with_state(&state, &result.query).unwrap();
        assert_eq!(again.prompt, result.prompt);
        assert_eq!(again.query, result.query);

        seen.insert(result.query);
    }

    assert!(
        seen.len() > 1,
        "expected multiple distinct random queries, got {seen:?}"
    );
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
