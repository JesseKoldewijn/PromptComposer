use std::collections::HashMap;
use std::path::{Path, PathBuf};

use calamine::{Data, Reader, Xlsx};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::ComposeError;

pub const MAX_OPERATIONAL_LEVEL: u8 = 5;
pub const MAX_INDEX: u8 = 30;
pub const ARCHIVE_FILENAME: &str = "prompt_archive.xlsx";
pub const STORE_FILENAME: &str = "settings.json";
pub const STORE_ARCHIVE_KEY: &str = "archive";

#[derive(Debug, Clone, Serialize)]
pub struct Subject {
    pub row: u32,
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryEntry {
    pub name: String,
    pub level: u8,
    pub index: u8,
    pub status: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Outfit,
    Pose,
    Action,
    Scene,
}

impl Category {
    pub fn sheet_name(self) -> &'static str {
        match self {
            Category::Outfit => "Outfits",
            Category::Pose => "Poses",
            Category::Action => "Actions",
            Category::Scene => "Scenes",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Category::Outfit => "Outfit",
            Category::Pose => "Pose",
            Category::Action => "Action",
            Category::Scene => "Scene",
        }
    }

    pub fn from_slot(slot: usize) -> Option<Self> {
        match slot {
            0 => Some(Category::Outfit),
            1 => Some(Category::Pose),
            2 => Some(Category::Action),
            3 => Some(Category::Scene),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Catalog {
    pub subjects: HashMap<u32, Subject>,
    pub outfits: HashMap<(u8, u8), CategoryEntry>,
    pub poses: HashMap<(u8, u8), CategoryEntry>,
    pub actions: HashMap<(u8, u8), CategoryEntry>,
    pub scenes: HashMap<(u8, u8), CategoryEntry>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CatalogCounts {
    pub subjects: usize,
    pub outfits: usize,
    pub poses: usize,
    pub actions: usize,
    pub scenes: usize,
}

impl Catalog {
    pub fn load(path: &Path) -> Result<Self, ComposeError> {
        let mut workbook: Xlsx<_> = calamine::open_workbook(path).map_err(|e| {
            ComposeError::Catalog(format!("failed to open workbook {}: {e}", path.display()))
        })?;

        let catalog = Catalog {
            subjects: load_subjects(&mut workbook)?,
            outfits: load_category(&mut workbook, Category::Outfit)?,
            poses: load_category(&mut workbook, Category::Pose)?,
            actions: load_category(&mut workbook, Category::Action)?,
            scenes: load_category(&mut workbook, Category::Scene)?,
        };

        if catalog.subjects.is_empty() {
            return Err(ComposeError::Catalog(
                "workbook has no subjects — is this a valid prompt archive?".into(),
            ));
        }

        Ok(catalog)
    }

    pub fn counts(&self) -> CatalogCounts {
        CatalogCounts {
            subjects: self.subjects.len(),
            outfits: self.outfits.len(),
            poses: self.poses.len(),
            actions: self.actions.len(),
            scenes: self.scenes.len(),
        }
    }

    pub fn subject(&self, row: u32) -> Result<&Subject, ComposeError> {
        self.subjects.get(&row).ok_or_else(|| {
            let max = self.subjects.keys().copied().max().unwrap_or(0);
            ComposeError::invalid(
                "subject_out_of_range",
                format!("subject row {row} not found (valid Excel rows: 2–{max})"),
            )
        })
    }

    pub fn entry(
        &self,
        category: Category,
        level: u8,
        index: u8,
    ) -> Result<&CategoryEntry, ComposeError> {
        let map = match category {
            Category::Outfit => &self.outfits,
            Category::Pose => &self.poses,
            Category::Action => &self.actions,
            Category::Scene => &self.scenes,
        };
        map.get(&(level, index)).ok_or_else(|| {
            ComposeError::invalid(
                "entry_not_found",
                format!(
                    "{} L{level}-{index:02} not found in {}",
                    category.label(),
                    category.sheet_name()
                ),
            )
        })
    }
}

pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, ComposeError> {
    app.path()
        .app_data_dir()
        .map_err(|e| ComposeError::Catalog(format!("app data dir: {e}")))
}

pub fn archive_path(app: &AppHandle) -> Result<PathBuf, ComposeError> {
    Ok(app_data_dir(app)?.join(ARCHIVE_FILENAME))
}

fn cell_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{e:?}"),
    }
}

fn load_subjects(
    workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
) -> Result<HashMap<u32, Subject>, ComposeError> {
    let range = workbook
        .worksheet_range("Subjects")
        .map_err(|e| ComposeError::Catalog(format!("Subjects sheet: {e}")))?;

    let mut subjects = HashMap::new();
    for (idx, row) in range.rows().enumerate() {
        let excel_row = (idx + 1) as u32;
        if excel_row == 1 || row.is_empty() {
            continue;
        }
        let name = cell_string(row.first().unwrap_or(&Data::Empty));
        if name.is_empty() {
            continue;
        }
        let body = cell_string(row.get(1).unwrap_or(&Data::Empty));
        if body.is_empty() {
            continue;
        }
        subjects.insert(
            excel_row,
            Subject {
                row: excel_row,
                name,
                body,
            },
        );
    }
    Ok(subjects)
}

fn parse_level_index(name: &str) -> Option<(u8, u8)> {
    let pos = name.rfind('L')?;
    let suffix = &name[pos + 1..];
    let (level_s, index_s) = suffix.split_once('-')?;
    let level: u8 = level_s.parse().ok()?;
    let index: u8 = index_s.parse().ok()?;
    Some((level, index))
}

fn load_category(
    workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
    category: Category,
) -> Result<HashMap<(u8, u8), CategoryEntry>, ComposeError> {
    let sheet = category.sheet_name();
    let range = workbook
        .worksheet_range(sheet)
        .map_err(|e| ComposeError::Catalog(format!("{sheet} sheet: {e}")))?;

    let mut map = HashMap::new();
    for (idx, row) in range.rows().enumerate() {
        if idx == 0 || row.is_empty() {
            continue;
        }
        let name = cell_string(row.first().unwrap_or(&Data::Empty));
        if name.is_empty() {
            continue;
        }
        let Some((level, index)) = parse_level_index(&name) else {
            continue;
        };
        let status = cell_string(row.get(2).unwrap_or(&Data::Empty));
        let prompt = cell_string(row.get(3).unwrap_or(&Data::Empty));
        if prompt.is_empty() {
            continue;
        }
        map.insert(
            (level, index),
            CategoryEntry {
                name,
                level,
                index,
                status,
                prompt,
            },
        );
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures_data::{
        self, fixture_path, ACTION_INDEX, ACTION_LEVEL, ACTION_PROMPT, OUTFIT_INDEX, OUTFIT_LEVEL,
        OUTFIT_PROMPT, SUBJECT_BODY, SUBJECT_NAME, SUBJECT_ROW,
    };

    #[test]
    fn loads_minimal_fixture() {
        let catalog = Catalog::load(&fixture_path()).unwrap();
        let counts = catalog.counts();
        assert_eq!(counts.subjects, 1);
        assert_eq!(counts.outfits, 1);
        assert_eq!(counts.poses, 1);
        assert_eq!(counts.actions, 1);
        assert_eq!(counts.scenes, 1);

        let subject = catalog.subject(SUBJECT_ROW).unwrap();
        assert_eq!(subject.name, SUBJECT_NAME);
        assert_eq!(subject.body, SUBJECT_BODY);

        assert_eq!(
            catalog
                .entry(Category::Outfit, OUTFIT_LEVEL, OUTFIT_INDEX)
                .unwrap()
                .prompt,
            OUTFIT_PROMPT
        );
        assert_eq!(
            catalog
                .entry(Category::Action, ACTION_LEVEL, ACTION_INDEX)
                .unwrap()
                .prompt,
            ACTION_PROMPT
        );
        let _ = fixtures_data::POSE_PROMPT;
    }
}
