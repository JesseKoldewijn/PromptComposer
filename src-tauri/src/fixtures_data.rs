//! Shared fixture constants — used by `gen_fixture`, unit tests, and integration tests.

pub const FIXTURE_RELATIVE: &str = "fixtures/minimal_prompt_archive.xlsx";

// --- Golden path (subject row 2 + L1-01 / L2-01 / L1-02) ---

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

pub const GOLDEN_QUERY_SLASH: &str = "2 1/1 2/1 1/2";

pub const OUTFIT_ONLY_QUERY: &str = "2 1lvl1";
pub const OUTFIT_ONLY_PROMPT: &str = "BODY_ALPHA OUTFIT_1_1";

// --- Alternate path (subject row 3 + max-level / other indexes) ---

pub const ALT_SUBJECT_ROW: u32 = 3;
pub const ALT_SUBJECT_NAME: &str = "Beta";
pub const ALT_SUBJECT_BODY: &str = "BODY_BETA";

pub const ALT_OUTFIT_NAME: &str = "Outfit L5-30";
pub const ALT_OUTFIT_LEVEL: u8 = 5;
pub const ALT_OUTFIT_INDEX: u8 = 30;
pub const ALT_OUTFIT_PROMPT: &str = "OUTFIT_5_30";

pub const ALT_POSE_NAME: &str = "Pose L1-01";
pub const ALT_POSE_LEVEL: u8 = 1;
pub const ALT_POSE_INDEX: u8 = 1;
pub const ALT_POSE_PROMPT: &str = "POSE_1_1";

pub const ALT_ACTION_NAME: &str = "Action L4-10";
pub const ALT_ACTION_LEVEL: u8 = 4;
pub const ALT_ACTION_INDEX: u8 = 10;
pub const ALT_ACTION_PROMPT: &str = "ACTION_4_10";

pub const ALT_SCENE_NAME: &str = "Scene L1-01";
pub const ALT_SCENE_LEVEL: u8 = 1;
pub const ALT_SCENE_INDEX: u8 = 1;
pub const ALT_SCENE_PROMPT: &str = "SCENE_1_1";

pub const ALT_QUERY: &str = "3 5lvl30 1lvl1 4lvl10";
pub const ALT_PROMPT: &str = "BODY_BETA OUTFIT_5_30 POSE_1_1 ACTION_4_10";

pub const ALT_QUERY_WITH_SCENE: &str = "3 5lvl30 1lvl1 4lvl10 1lvl1";
pub const ALT_PROMPT_WITH_SCENE: &str = "BODY_BETA OUTFIT_5_30 POSE_1_1 ACTION_4_10 SCENE_1_1";

/// Extra mid-range keys so “entry not found” can target a valid parse that misses the sheet.
pub const EXTRA_OUTFIT_NAME: &str = "Outfit L2-02";
pub const EXTRA_OUTFIT_LEVEL: u8 = 2;
pub const EXTRA_OUTFIT_INDEX: u8 = 2;
pub const EXTRA_OUTFIT_PROMPT: &str = "OUTFIT_2_2";

pub const EXTRA_POSE_NAME: &str = "Pose L3-05";
pub const EXTRA_POSE_LEVEL: u8 = 3;
pub const EXTRA_POSE_INDEX: u8 = 5;
pub const EXTRA_POSE_PROMPT: &str = "POSE_3_5";

pub const EXTRA_ACTION_NAME: &str = "Action L2-03";
pub const EXTRA_ACTION_LEVEL: u8 = 2;
pub const EXTRA_ACTION_INDEX: u8 = 3;
pub const EXTRA_ACTION_PROMPT: &str = "ACTION_2_3";

pub const EXTRA_SCENE_NAME: &str = "Scene L5-15";
pub const EXTRA_SCENE_LEVEL: u8 = 5;
pub const EXTRA_SCENE_INDEX: u8 = 15;
pub const EXTRA_SCENE_PROMPT: &str = "SCENE_5_15";

pub const FIXTURE_SUBJECT_COUNT: usize = 2;
pub const FIXTURE_OUTFIT_COUNT: usize = 3;
pub const FIXTURE_POSE_COUNT: usize = 3;
pub const FIXTURE_ACTION_COUNT: usize = 3;
pub const FIXTURE_SCENE_COUNT: usize = 3;

/// Category rows written into the fixture (Name, Level, Prompt).
pub type CategoryRow = (&'static str, u8, &'static str);

pub const FIXTURE_OUTFITS: &[CategoryRow] = &[
    (OUTFIT_NAME, OUTFIT_LEVEL, OUTFIT_PROMPT),
    (ALT_OUTFIT_NAME, ALT_OUTFIT_LEVEL, ALT_OUTFIT_PROMPT),
    (EXTRA_OUTFIT_NAME, EXTRA_OUTFIT_LEVEL, EXTRA_OUTFIT_PROMPT),
];

pub const FIXTURE_POSES: &[CategoryRow] = &[
    (POSE_NAME, POSE_LEVEL, POSE_PROMPT),
    (ALT_POSE_NAME, ALT_POSE_LEVEL, ALT_POSE_PROMPT),
    (EXTRA_POSE_NAME, EXTRA_POSE_LEVEL, EXTRA_POSE_PROMPT),
];

pub const FIXTURE_ACTIONS: &[CategoryRow] = &[
    (ACTION_NAME, ACTION_LEVEL, ACTION_PROMPT),
    (ALT_ACTION_NAME, ALT_ACTION_LEVEL, ALT_ACTION_PROMPT),
    (EXTRA_ACTION_NAME, EXTRA_ACTION_LEVEL, EXTRA_ACTION_PROMPT),
];

pub const FIXTURE_SCENES: &[CategoryRow] = &[
    (SCENE_NAME, SCENE_LEVEL, SCENE_PROMPT),
    (ALT_SCENE_NAME, ALT_SCENE_LEVEL, ALT_SCENE_PROMPT),
    (EXTRA_SCENE_NAME, EXTRA_SCENE_LEVEL, EXTRA_SCENE_PROMPT),
];

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
