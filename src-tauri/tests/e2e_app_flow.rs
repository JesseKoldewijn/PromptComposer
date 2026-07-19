//! End-to-end flow through the same archive/compose APIs the UI invokes.
//! (GUI WebdriverIO specs live under /e2e; this covers the full backend path.)

use app_lib::archive::{
    clear_archive_in, compose_with_state, import_archive_into, load_state_from_dir, AppState,
};
use app_lib::fixtures_data::{fixture_path, ALT_PROMPT, ALT_QUERY, GOLDEN_PROMPT, GOLDEN_QUERY};
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
