//! Canonical prompt-archive sheet/header contract.
//!
//! Derived from the production archive layout (Subjects/Maidens + category sheets).
//! Tests and writers use these constants — never a path to a local master workbook.

use std::path::Path;

use calamine::{Data, Reader, Xlsx};

use crate::error::ComposeError;

/// Canonical subject sheet name (writers always use this).
pub const SUBJECT_SHEET_CANONICAL: &str = "Subjects";

/// Legacy subject sheet name still accepted by the loader.
pub const SUBJECT_SHEET_LEGACY: &str = "Maidens";

/// Subject sheet candidates in preference order (canonical first).
pub const SUBJECT_SHEET_CANDIDATES: &[&str] = &[SUBJECT_SHEET_CANONICAL, SUBJECT_SHEET_LEGACY];

/// Required category sheets (exact names).
pub const CATEGORY_SHEETS: &[&str] = &["Outfits", "Poses", "Actions", "Scenes"];

/// Header row for Subjects / Maidens.
pub const SUBJECT_HEADERS: &[&str] = &["Name", "Body", "Outfit", "Accessories"];

/// Header row for Outfits / Poses / Actions / Scenes.
pub const CATEGORY_HEADERS: &[&str] = &["Name", "Level", "Status", "Prompt"];

/// Optional documentation sheet written by the blank-archive template.
pub const HOW_TO_SHEET: &str = "HowTo";

/// Options for [`assert_archive_shape`].
#[derive(Debug, Clone, Copy)]
pub struct ShapeOptions {
    /// Require a `HowTo` sheet (template archives).
    pub require_howto: bool,
    /// Allow legacy `Maidens` as the subjects sheet (fixture/loadable archives).
    pub allow_legacy_subjects: bool,
}

impl Default for ShapeOptions {
    fn default() -> Self {
        Self {
            require_howto: false,
            allow_legacy_subjects: true,
        }
    }
}

impl ShapeOptions {
    /// Shape expected of the e2e / unit-test fixture.
    pub fn fixture() -> Self {
        Self {
            require_howto: false,
            allow_legacy_subjects: true,
        }
    }

    /// Shape expected of [`crate::workbook::write_template_archive`] output.
    pub fn template() -> Self {
        Self {
            require_howto: true,
            allow_legacy_subjects: false,
        }
    }
}

/// Assert that `path` has the required sheets and header rows.
///
/// Extra sheets are allowed. Status cell values are not checked.
pub fn assert_archive_shape(path: &Path, options: ShapeOptions) -> Result<(), ComposeError> {
    let mut workbook: Xlsx<_> = calamine::open_workbook(path)
        .map_err(|e| ComposeError::Catalog(format!("open workbook for shape check: {e}")))?;

    let sheet_names = workbook.sheet_names().to_vec();

    let subjects_sheet = resolve_subjects_sheet(&sheet_names, options.allow_legacy_subjects)?;
    assert_headers(&mut workbook, subjects_sheet, SUBJECT_HEADERS)?;

    for &sheet in CATEGORY_SHEETS {
        if !sheet_names.iter().any(|n| n == sheet) {
            return Err(ComposeError::Catalog(format!(
                "missing required sheet `{sheet}`"
            )));
        }
        assert_headers(&mut workbook, sheet, CATEGORY_HEADERS)?;
    }

    if options.require_howto && !sheet_names.iter().any(|n| n == HOW_TO_SHEET) {
        return Err(ComposeError::Catalog(format!(
            "missing required sheet `{HOW_TO_SHEET}`"
        )));
    }

    Ok(())
}

fn resolve_subjects_sheet(
    sheet_names: &[String],
    allow_legacy: bool,
) -> Result<&str, ComposeError> {
    let has_canonical = sheet_names.iter().any(|n| n == SUBJECT_SHEET_CANONICAL);
    let has_legacy = sheet_names.iter().any(|n| n == SUBJECT_SHEET_LEGACY);

    if has_canonical {
        return Ok(SUBJECT_SHEET_CANONICAL);
    }
    if allow_legacy && has_legacy {
        return Ok(SUBJECT_SHEET_LEGACY);
    }
    if has_legacy && !allow_legacy {
        return Err(ComposeError::Catalog(format!(
            "found legacy sheet `{SUBJECT_SHEET_LEGACY}` but canonical `{SUBJECT_SHEET_CANONICAL}` is required"
        )));
    }
    Err(ComposeError::Catalog(format!(
        "missing subjects sheet (`{SUBJECT_SHEET_CANONICAL}` or `{SUBJECT_SHEET_LEGACY}`)"
    )))
}

fn assert_headers(
    workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
    sheet: &str,
    expected: &[&str],
) -> Result<(), ComposeError> {
    let range = workbook
        .worksheet_range(sheet)
        .map_err(|e| ComposeError::Catalog(format!("read sheet `{sheet}` for shape check: {e}")))?;

    let Some(header_row) = range.rows().next() else {
        return Err(ComposeError::Catalog(format!(
            "sheet `{sheet}` has no header row"
        )));
    };

    for (i, &expected_header) in expected.iter().enumerate() {
        let actual = header_row
            .get(i)
            .map(header_cell_string)
            .unwrap_or_default();
        if actual != expected_header {
            return Err(ComposeError::Catalog(format!(
                "sheet `{sheet}` header col {i}: expected `{expected_header}`, got `{actual}`"
            )));
        }
    }
    Ok(())
}

fn header_cell_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.trim().to_string(),
        Data::Error(e) => format!("{e:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures_data;
    use crate::workbook::write_template_archive;
    use rust_xlsxwriter::Workbook;
    use tempfile::tempdir;

    #[test]
    fn e2e_fixture_matches_archive_schema() {
        let path = fixtures_data::fixture_path();
        assert!(
            path.is_file(),
            "fixture missing at {}; run `npm run fixture:gen`",
            path.display()
        );
        assert_archive_shape(&path, ShapeOptions::fixture()).unwrap();
    }

    #[test]
    fn template_matches_archive_schema() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("template.xlsx");
        write_template_archive(&path).unwrap();
        assert_archive_shape(&path, ShapeOptions::template()).unwrap();
    }

    #[test]
    fn wrong_headers_fail_shape_check() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.xlsx");

        let mut workbook = Workbook::new();
        {
            let sheet = workbook
                .add_worksheet()
                .set_name(SUBJECT_SHEET_CANONICAL)
                .unwrap();
            sheet.write_string(0, 0, "Wrong").unwrap();
            sheet.write_string(0, 1, "Headers").unwrap();
            sheet.write_string(0, 2, "Here").unwrap();
            sheet.write_string(0, 3, "Now").unwrap();
        }
        for &name in CATEGORY_SHEETS {
            let sheet = workbook.add_worksheet().set_name(name).unwrap();
            for (i, &h) in CATEGORY_HEADERS.iter().enumerate() {
                sheet.write_string(0, i as u16, h).unwrap();
            }
        }
        workbook.save(&path).unwrap();

        let err = assert_archive_shape(&path, ShapeOptions::fixture()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("header"),
            "expected header mismatch, got: {msg}"
        );
    }

    #[test]
    fn missing_category_sheet_fails_shape_check() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("incomplete.xlsx");

        let mut workbook = Workbook::new();
        {
            let sheet = workbook
                .add_worksheet()
                .set_name(SUBJECT_SHEET_CANONICAL)
                .unwrap();
            for (i, &h) in SUBJECT_HEADERS.iter().enumerate() {
                sheet.write_string(0, i as u16, h).unwrap();
            }
        }
        // Only Outfits — omit Poses/Actions/Scenes
        {
            let sheet = workbook.add_worksheet().set_name("Outfits").unwrap();
            for (i, &h) in CATEGORY_HEADERS.iter().enumerate() {
                sheet.write_string(0, i as u16, h).unwrap();
            }
        }
        workbook.save(&path).unwrap();

        let err = assert_archive_shape(&path, ShapeOptions::fixture()).unwrap_err();
        assert!(err.to_string().contains("Poses"));
    }
}
