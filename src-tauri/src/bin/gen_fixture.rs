use std::path::PathBuf;

use app_lib::archive_schema::{
    CATEGORY_HEADERS, CATEGORY_SHEETS, SUBJECT_HEADERS, SUBJECT_SHEET_CANONICAL,
};
use app_lib::fixtures_data::{
    self, CategoryRow, ALT_SUBJECT_BODY, ALT_SUBJECT_NAME, FIXTURE_ACTIONS, FIXTURE_OUTFITS,
    FIXTURE_POSES, FIXTURE_SCENES, SUBJECT_BODY, SUBJECT_NAME,
};
use rust_xlsxwriter::{Workbook, Worksheet, XlsxError};

fn main() -> Result<(), XlsxError> {
    let out = resolve_out_path();
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create fixtures dir");
    }

    let mut workbook = Workbook::new();

    {
        let sheet = workbook.add_worksheet().set_name(SUBJECT_SHEET_CANONICAL)?;
        for (i, &header) in SUBJECT_HEADERS.iter().enumerate() {
            sheet.write_string(0, i as u16, header)?;
        }
        // Excel row 2 → query token `2`
        sheet.write_string(1, 0, SUBJECT_NAME)?;
        sheet.write_string(1, 1, SUBJECT_BODY)?;
        // Excel row 3 → query token `3`
        sheet.write_string(2, 0, ALT_SUBJECT_NAME)?;
        sheet.write_string(2, 1, ALT_SUBJECT_BODY)?;
    }

    write_category_sheet(&mut workbook, CATEGORY_SHEETS[0], FIXTURE_OUTFITS)?;
    write_category_sheet(&mut workbook, CATEGORY_SHEETS[1], FIXTURE_POSES)?;
    write_category_sheet(&mut workbook, CATEGORY_SHEETS[2], FIXTURE_ACTIONS)?;
    write_category_sheet(&mut workbook, CATEGORY_SHEETS[3], FIXTURE_SCENES)?;

    workbook.save(&out)?;
    println!("wrote {}", out.display());
    println!("golden query: {}", fixtures_data::GOLDEN_QUERY);
    println!("golden prompt: {}", fixtures_data::GOLDEN_PROMPT);
    println!("alt query: {}", fixtures_data::ALT_QUERY);
    println!("alt prompt: {}", fixtures_data::ALT_PROMPT);
    Ok(())
}

fn write_category_sheet(
    workbook: &mut Workbook,
    sheet_name: &str,
    rows: &[CategoryRow],
) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet().set_name(sheet_name)?;
    for (i, &header) in CATEGORY_HEADERS.iter().enumerate() {
        sheet.write_string(0, i as u16, header)?;
    }
    for (row_idx, &(name, level, prompt)) in rows.iter().enumerate() {
        write_category_row(sheet, (row_idx + 1) as u32, name, level, prompt)?;
    }
    Ok(())
}

fn write_category_row(
    sheet: &mut Worksheet,
    row: u32,
    name: &str,
    level: u8,
    prompt: &str,
) -> Result<(), XlsxError> {
    sheet.write_string(row, 0, name)?;
    sheet.write_number(row, 1, f64::from(level))?;
    sheet.write_string(row, 2, "USE")?;
    sheet.write_string(row, 3, prompt)?;
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
