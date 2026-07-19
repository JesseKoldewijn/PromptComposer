use std::path::PathBuf;

use app_lib::fixtures_data::{
    self, ACTION_NAME, ACTION_PROMPT, OUTFIT_NAME, OUTFIT_PROMPT, POSE_NAME, POSE_PROMPT,
    SCENE_NAME, SCENE_PROMPT, SUBJECT_BODY, SUBJECT_NAME,
};
use rust_xlsxwriter::{Workbook, XlsxError};

fn main() -> Result<(), XlsxError> {
    let out = resolve_out_path();
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create fixtures dir");
    }

    let mut workbook = Workbook::new();

    {
        let sheet = workbook.add_worksheet().set_name("Subjects")?;
        sheet.write_string(0, 0, "Name")?;
        sheet.write_string(0, 1, "Body")?;
        sheet.write_string(0, 2, "Outfit")?;
        sheet.write_string(0, 3, "Accessories")?;
        sheet.write_string(1, 0, SUBJECT_NAME)?;
        sheet.write_string(1, 1, SUBJECT_BODY)?;
    }

    write_category(&mut workbook, "Outfits", OUTFIT_NAME, 1, OUTFIT_PROMPT)?;
    write_category(&mut workbook, "Poses", POSE_NAME, 2, POSE_PROMPT)?;
    write_category(&mut workbook, "Actions", ACTION_NAME, 1, ACTION_PROMPT)?;
    write_category(&mut workbook, "Scenes", SCENE_NAME, 3, SCENE_PROMPT)?;

    workbook.save(&out)?;
    println!("wrote {}", out.display());
    println!("golden query: {}", fixtures_data::GOLDEN_QUERY);
    println!("golden prompt: {}", fixtures_data::GOLDEN_PROMPT);
    Ok(())
}

fn write_category(
    workbook: &mut Workbook,
    sheet_name: &str,
    entry_name: &str,
    level: u8,
    prompt: &str,
) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet().set_name(sheet_name)?;
    sheet.write_string(0, 0, "Name")?;
    sheet.write_string(0, 1, "Level")?;
    sheet.write_string(0, 2, "Status")?;
    sheet.write_string(0, 3, "Prompt")?;
    sheet.write_string(1, 0, entry_name)?;
    sheet.write_number(1, 1, level as f64)?;
    sheet.write_string(1, 2, "USE")?;
    sheet.write_string(1, 3, prompt)?;
    Ok(())
}

fn resolve_out_path() -> PathBuf {
    // Prefer repo-root fixtures/ (npm run from root) or ../fixtures (cargo from src-tauri).
    let candidates = [
        (
            PathBuf::from("package.json"),
            PathBuf::from("fixtures/minimal_prompt_archive.xlsx"),
        ),
        (
            PathBuf::from("../package.json"),
            PathBuf::from("../fixtures/minimal_prompt_archive.xlsx"),
        ),
    ];
    for (marker, out) in &candidates {
        if marker.is_file() {
            if let Some(parent) = out.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            return out.clone();
        }
    }
    fixtures_data::fixture_path()
}
