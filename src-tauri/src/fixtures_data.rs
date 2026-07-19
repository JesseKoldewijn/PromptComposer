//! Shared fixture constants — used by `gen_fixture`, unit tests, and integration tests.

pub const FIXTURE_RELATIVE: &str = "fixtures/minimal_prompt_archive.xlsx";

pub const SUBJECT_ROW: u32 = 2;
pub const SUBJECT_NAME: &str = "Alpha";
pub const SUBJECT_BODY: &str = "BODY_ALPHA";

pub const OUTFIT_NAME: &str = "Outfit L1-01";
pub const OUTFIT_LEVEL: u8 = 1;
pub const OUTFIT_INDEX: u8 = 1;
pub const OUTFIT_PROMPT: &str = "OUTFIT_1_1";

pub const POSE_NAME: &str = "Pose L2-01";
pub const POSE_LEVEL: u8 = 2;
pub const POSE_INDEX: u8 = 1;
pub const POSE_PROMPT: &str = "POSE_2_1";

pub const ACTION_NAME: &str = "Action L1-02";
pub const ACTION_LEVEL: u8 = 1;
pub const ACTION_INDEX: u8 = 2;
pub const ACTION_PROMPT: &str = "ACTION_1_2";

pub const SCENE_NAME: &str = "Scene L3-01";
pub const SCENE_LEVEL: u8 = 3;
pub const SCENE_INDEX: u8 = 1;
pub const SCENE_PROMPT: &str = "SCENE_3_1";

pub const GOLDEN_QUERY: &str = "2 1lvl1 2lvl1 1lvl2";
pub const GOLDEN_PROMPT: &str = "BODY_ALPHA OUTFIT_1_1 POSE_2_1 ACTION_1_2";

pub const GOLDEN_QUERY_WITH_SCENE: &str = "2 1lvl1 2lvl1 1lvl2 3lvl1";
pub const GOLDEN_PROMPT_WITH_SCENE: &str = "BODY_ALPHA OUTFIT_1_1 POSE_2_1 ACTION_1_2 SCENE_3_1";

use std::path::PathBuf;

/// Resolve the checked-in fixture from common working directories.
pub fn fixture_path() -> PathBuf {
    let candidates = [
        PathBuf::from(FIXTURE_RELATIVE),
        PathBuf::from("..").join(FIXTURE_RELATIVE),
        PathBuf::from("../..").join(FIXTURE_RELATIVE),
        PathBuf::from("src-tauri/../").join(FIXTURE_RELATIVE),
    ];
    for p in candidates {
        if p.is_file() {
            return p;
        }
    }
    // Default write/read location when generating from src-tauri/
    PathBuf::from("..").join(FIXTURE_RELATIVE)
}
