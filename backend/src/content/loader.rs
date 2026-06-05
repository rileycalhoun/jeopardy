use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::content::models::{QuestionPack, QuestionPackSummary};

#[derive(Clone, Debug)]
pub struct QuestionPackLoader {
    root: PathBuf,
}

impl QuestionPackLoader {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn list(&self) -> Result<Vec<QuestionPackSummary>, String> {
        let mut packs = self.load_all()?;
        packs.sort_by(|left, right| left.title.cmp(&right.title));
        Ok(packs.into_iter().map(|pack| pack.summary()).collect())
    }

    pub fn load(&self, pack_id: &str) -> Result<QuestionPack, String> {
        self.load_all()?
            .into_iter()
            .find(|pack| pack.id == pack_id)
            .ok_or_else(|| format!("question pack '{pack_id}' was not found"))
    }

    pub fn load_all(&self) -> Result<Vec<QuestionPack>, String> {
        let entries = fs::read_dir(&self.root)
            .map_err(|err| format!("could not read question pack directory: {err}"))?;
        let mut packs = Vec::new();
        let mut ids = HashSet::new();

        for entry in entries {
            let entry =
                entry.map_err(|err| format!("could not read question pack entry: {err}"))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let pack = load_pack_file(&path)?;
            if !ids.insert(pack.id.clone()) {
                return Err(format!("duplicate question pack id '{}'", pack.id));
            }
            packs.push(pack);
        }

        Ok(packs)
    }
}

fn load_pack_file(path: &Path) -> Result<QuestionPack, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("could not read question pack '{}': {err}", path.display()))?;
    let pack: QuestionPack = serde_json::from_str(&raw)
        .map_err(|err| format!("could not parse question pack '{}': {err}", path.display()))?;
    pack.validate()
        .map_err(|err| format!("invalid question pack '{}': {err}", path.display()))?;
    Ok(pack)
}
