use app_lib::archive::{
    clear_archive_in, compose_with_state, import_archive_into, load_state_from_dir, AppState,
};
use app_lib::fixtures_data::{
    fixture_path, ALT_PROMPT, ALT_QUERY, GOLDEN_PROMPT, GOLDEN_PROMPT_WITH_SCENE, GOLDEN_QUERY,
    GOLDEN_QUERY_WITH_SCENE,
};
use tempfile::tempdir;

#[test]
fn archive_lifecycle_import_compose_clear() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path();
    let fixture = fixture_path();

    let mut state = AppState::default();
    let err = compose_with_state(&state, GOLDEN_QUERY).unwrap_err();
    assert!(err.to_string().contains("no archive") || format!("{err:?}").contains("no_archive"));

    let status = import_archive_into(&mut state, &fixture, data_dir).unwrap();
    assert!(status.loaded);
    assert_eq!(
        status.original_name.as_deref(),
        Some("minimal_prompt_archive.xlsx")
    );
    let counts = status.counts.expect("counts");
    assert_eq!(counts.subjects, 2);
    assert_eq!(counts.outfits, 3);

    let ranges = status.ranges.expect("ranges");
    assert_eq!(ranges.subjects.min_row, 2);
    assert_eq!(ranges.subjects.max_row, 3);
    let outfits = ranges.outfits.expect("outfit ranges");
    assert_eq!(outfits.min_level, 1);
    assert_eq!(outfits.max_level, 5);
    assert_eq!(outfits.min_index, 1);
    assert_eq!(outfits.max_index, 30);

    let result = compose_with_state(&state, GOLDEN_QUERY).unwrap();
    assert_eq!(result.prompt, GOLDEN_PROMPT);

    let with_scene = compose_with_state(&state, GOLDEN_QUERY_WITH_SCENE).unwrap();
    assert_eq!(with_scene.prompt, GOLDEN_PROMPT_WITH_SCENE);

    let alt = compose_with_state(&state, ALT_QUERY).unwrap();
    assert_eq!(alt.prompt, ALT_PROMPT);

    let cleared = clear_archive_in(&mut state, data_dir).unwrap();
    assert!(!cleared.loaded);
    assert!(compose_with_state(&state, GOLDEN_QUERY).is_err());

    let again = import_archive_into(&mut state, &fixture, data_dir).unwrap();
    assert!(again.loaded);
    assert_eq!(
        compose_with_state(&state, GOLDEN_QUERY).unwrap().prompt,
        GOLDEN_PROMPT
    );
}

#[test]
fn reload_from_dir_after_import() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path();
    let mut state = AppState::default();
    import_archive_into(&mut state, &fixture_path(), data_dir).unwrap();

    let reloaded = load_state_from_dir(data_dir).unwrap();
    assert!(reloaded.catalog.is_some());
    assert_eq!(
        compose_with_state(&reloaded, GOLDEN_QUERY).unwrap().prompt,
        GOLDEN_PROMPT
    );
}
