use app_lib::archive::{
    clear_archive_in, compose_with_state, import_archive_into, load_state_from_dir, AppState,
};
use app_lib::fixtures_data::{
    fixture_path, GOLDEN_PROMPT, GOLDEN_PROMPT_WITH_SCENE, GOLDEN_QUERY, GOLDEN_QUERY_WITH_SCENE,
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
    assert_eq!(counts.subjects, 1);
    assert_eq!(counts.outfits, 1);

    let result = compose_with_state(&state, GOLDEN_QUERY).unwrap();
    assert_eq!(result.prompt, GOLDEN_PROMPT);

    let with_scene = compose_with_state(&state, GOLDEN_QUERY_WITH_SCENE).unwrap();
    assert_eq!(with_scene.prompt, GOLDEN_PROMPT_WITH_SCENE);

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
