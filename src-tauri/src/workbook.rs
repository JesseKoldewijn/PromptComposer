//! Build prompt-archive workbooks (template + test fixture layout).

use std::path::Path;

use rust_xlsxwriter::{Workbook, XlsxError};

use crate::archive_schema::{
    CATEGORY_HEADERS, CATEGORY_SHEETS, HOW_TO_SHEET, SUBJECT_HEADERS, SUBJECT_SHEET_CANONICAL,
};
use crate::error::ComposeError;

/// Write a starter archive with the sheets/columns this app expects.
/// Sample cells use `e.g.` hint text so users know what to replace.
pub fn write_template_archive(path: &Path) -> Result<(), ComposeError> {
    let mut workbook = Workbook::new();

    {
        let sheet = workbook
            .add_worksheet()
            .set_name(SUBJECT_SHEET_CANONICAL)
            .map_err(xlsx_err)?;
        for (i, &header) in SUBJECT_HEADERS.iter().enumerate() {
            sheet.write_string(0, i as u16, header).map_err(xlsx_err)?;
        }
        // Excel row 2 → query token `2`
        sheet
            .write_string(1, 0, "e.g. Alice (display name)")
            .map_err(xlsx_err)?;
        sheet
            .write_string(
                1,
                1,
                "e.g. 1girl, long hair, blue eyes — full body/character prompt",
            )
            .map_err(xlsx_err)?;
        sheet
            .write_string(1, 2, "e.g. optional notes (ignored by composer)")
            .map_err(xlsx_err)?;
        sheet
            .write_string(1, 3, "e.g. optional notes (ignored by composer)")
            .map_err(xlsx_err)?;
    }

    let category_samples = [
        (
            CATEGORY_SHEETS[0],
            "Outfit L1-01",
            1u8,
            "e.g. wearing school uniform, white blouse — outfit prompt for 1lvl1",
        ),
        (
            CATEGORY_SHEETS[1],
            "Pose L2-01",
            2,
            "e.g. standing, looking at viewer — pose prompt for 2lvl1",
        ),
        (
            CATEGORY_SHEETS[2],
            "Action L1-02",
            1,
            "e.g. holding a book, slight smile — action prompt for 1lvl2",
        ),
        (
            CATEGORY_SHEETS[3],
            "Scene L3-01",
            3,
            "e.g. classroom interior, soft daylight — optional scene for 3lvl1",
        ),
    ];
    for (sheet_name, entry_name, level, prompt_hint) in category_samples {
        write_category_sheet(&mut workbook, sheet_name, entry_name, level, prompt_hint)?;
    }

    {
        let sheet = workbook
            .add_worksheet()
            .set_name(HOW_TO_SHEET)
            .map_err(xlsx_err)?;
        let lines = [
            "Prompt Composer archive template",
            "",
            "Sheets Subjects / Outfits / Poses / Actions / Scenes are required.",
            "HowTo is ignored by the app — safe to delete.",
            "",
            "Subjects: Excel row number is the query id (row 2 → query starts with 2).",
            "Only Name + Body are used; Body is the character prompt text.",
            "",
            "Outfits/Poses/Actions/Scenes:",
            "  Name MUST end with L{level}-{index} e.g. Outfit L1-01 → query 1lvl1",
            "  Level column is informational; Status USE marks an active row",
            "  Prompt is the text joined into the composed output",
            "",
            "Query example: 2 1lvl1 2lvl1 1lvl2",
            "  → Subject row 2 Body + Outfit L1-01 + Pose L2-01 + Action L1-02",
            "Optional scene: … 3lvl1 appends Scene L3-01",
            "",
            "Replace every e.g. hint with your own text, then Upload archive in the app.",
        ];
        for (i, line) in lines.iter().enumerate() {
            sheet.write_string(i as u32, 0, *line).map_err(xlsx_err)?;
        }
    }

    workbook.save(path).map_err(xlsx_err)?;
    Ok(())
}

fn write_category_sheet(
    workbook: &mut Workbook,
    sheet_name: &str,
    entry_name: &str,
    level: u8,
    prompt_hint: &str,
) -> Result<(), ComposeError> {
    let sheet = workbook
        .add_worksheet()
        .set_name(sheet_name)
        .map_err(xlsx_err)?;
    for (i, &header) in CATEGORY_HEADERS.iter().enumerate() {
        sheet.write_string(0, i as u16, header).map_err(xlsx_err)?;
    }
    sheet.write_string(1, 0, entry_name).map_err(xlsx_err)?;
    sheet
        .write_number(1, 1, f64::from(level))
        .map_err(xlsx_err)?;
    sheet.write_string(1, 2, "USE").map_err(xlsx_err)?;
    sheet.write_string(1, 3, prompt_hint).map_err(xlsx_err)?;
    Ok(())
}

fn xlsx_err(e: XlsxError) -> ComposeError {
    ComposeError::Catalog(format!("xlsx write failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use tempfile::tempdir;

    #[test]
    fn template_loads_as_catalog() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("template.xlsx");
        write_template_archive(&path).unwrap();
        let catalog = Catalog::load(&path).unwrap();
        let counts = catalog.counts();
        assert_eq!(counts.subjects, 1);
        assert_eq!(counts.outfits, 1);
        assert_eq!(counts.poses, 1);
        assert_eq!(counts.actions, 1);
        assert_eq!(counts.scenes, 1);
        assert!(catalog.subject(2).unwrap().body.contains("e.g."));
        assert!(catalog
            .entry(crate::catalog::Category::Outfit, 1, 1)
            .unwrap()
            .prompt
            .contains("e.g."));
    }
}
