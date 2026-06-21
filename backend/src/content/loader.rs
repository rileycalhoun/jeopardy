use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use tracing::{info, warn};

use crate::content::models::{CategoryPack, CategorySummary};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InvalidCategoryPolicy {
    Fail,
    Skip,
}

#[derive(Clone, Debug)]
pub struct CategoryPackLoader {
    root: PathBuf,
    invalid_policy: InvalidCategoryPolicy,
}

impl CategoryPackLoader {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            invalid_policy: invalid_category_policy_from_env(),
        }
    }

    #[cfg(test)]
    pub fn strict(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            invalid_policy: InvalidCategoryPolicy::Fail,
        }
    }

    #[cfg(test)]
    pub fn production(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            invalid_policy: InvalidCategoryPolicy::Skip,
        }
    }

    pub fn list(&self) -> Result<Vec<CategorySummary>, String> {
        let mut categories = self.load_all()?;
        categories.sort_by(|left, right| left.title.cmp(&right.title));
        Ok(categories
            .into_iter()
            .map(|category| category.summary())
            .collect())
    }

    pub fn load_selected(&self, category_ids: &[String]) -> Result<Vec<CategoryPack>, String> {
        if category_ids.is_empty() {
            return Err("at least one category must be selected".to_owned());
        }

        let selected_ids = category_ids.iter().collect::<HashSet<_>>();
        if selected_ids.len() != category_ids.len() {
            return Err("category ids must not be selected more than once".to_owned());
        }

        let categories = self.load_all()?;
        let mut selected = Vec::with_capacity(category_ids.len());
        for category_id in category_ids {
            let category = categories
                .iter()
                .find(|category| category.id == *category_id)
                .ok_or_else(|| format!("category '{category_id}' was not found"))?;
            selected.push(category.clone());
        }

        Ok(selected)
    }

    pub fn load_all(&self) -> Result<Vec<CategoryPack>, String> {
        let entries = fs::read_dir(&self.root)
            .map_err(|err| format!("could not read category directory: {err}"))?;
        let mut categories = Vec::new();
        let mut ids = HashSet::new();
        let mut files_found = 0_usize;
        let mut skipped = 0_usize;

        for entry in entries {
            let entry = entry.map_err(|err| format!("could not read category entry: {err}"))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            files_found += 1;

            match load_category_file(&path) {
                Ok(category) => {
                    if !ids.insert(category.id.clone()) {
                        let message = format!("duplicate category id '{}'", category.id);
                        if self.handle_invalid_category(&path, &message)? {
                            skipped += 1;
                            continue;
                        }
                    }
                    categories.push(category);
                }
                Err(message) => {
                    if self.handle_invalid_category(&path, &message)? {
                        skipped += 1;
                    }
                }
            }
        }

        info!(
            category_files_found = files_found,
            categories_loaded = categories.len(),
            categories_skipped = skipped,
            category_dir = %self.root.display(),
            "category loading summary"
        );

        Ok(categories)
    }

    fn handle_invalid_category(&self, path: &Path, reason: &str) -> Result<bool, String> {
        match self.invalid_policy {
            InvalidCategoryPolicy::Fail => Err(format!(
                "invalid category pack '{}': {reason}",
                path.display()
            )),
            InvalidCategoryPolicy::Skip => {
                warn!(
                    category_file = %path.display(),
                    reason,
                    "skipping invalid category pack"
                );
                Ok(true)
            }
        }
    }
}

fn load_category_file(path: &Path) -> Result<CategoryPack, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("could not read category pack '{}': {err}", path.display()))?;
    let category: CategoryPack = serde_json::from_str(&raw).map_err(|err| {
        format!(
            "malformed JSON in category pack '{}': {err}",
            path.display()
        )
    })?;
    category
        .validate()
        .map_err(|err| format!("validation failed: {err}"))?;
    Ok(category)
}

fn invalid_category_policy_from_env() -> InvalidCategoryPolicy {
    let is_production = ["APP_ENV", "RUST_ENV", "ENV"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .any(|value| value.eq_ignore_ascii_case("production"));

    if is_production {
        InvalidCategoryPolicy::Skip
    } else {
        InvalidCategoryPolicy::Fail
    }
}

#[cfg(test)]
mod category_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_category_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("jeopardy-category-loader-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp category dir should be created");
        dir
    }

    fn valid_category_json(id: &str, title: &str) -> String {
        format!(
            r#"{{
                "id": "{id}",
                "title": "{title}",
                "description": "Example category",
                "questions": [
                    {{ "points": 200, "question": "200?", "answer": "200!" }},
                    {{ "points": 400, "question": "400?", "answer": "400!" }},
                    {{ "points": 600, "question": "600?", "answer": "600!" }},
                    {{ "points": 800, "question": "800?", "answer": "800!" }},
                    {{ "points": 1000, "question": "1000?", "answer": "1000!" }}
                ]
            }}"#
        )
    }

    #[test]
    fn strict_loader_fails_loudly_on_invalid_category_files() {
        let dir = temp_category_dir("strict-invalid");
        fs::write(
            dir.join("broken.json"),
            r#"{ "id": "broken", "title": "Broken" }"#,
        )
        .expect("invalid category should be written");

        let loader = CategoryPackLoader::strict(&dir);
        let err = loader
            .load_all()
            .expect_err("strict mode should fail on invalid category");

        assert!(err.contains("invalid category pack"));
    }

    #[test]
    fn production_loader_skips_invalid_categories_and_keeps_valid_ones() {
        let dir = temp_category_dir("production-skip");
        fs::write(
            dir.join("movies.json"),
            valid_category_json("movies", "Movies"),
        )
        .expect("valid category should be written");
        fs::write(dir.join("broken.json"), "{").expect("malformed category should be written");

        let loader = CategoryPackLoader::production(&dir);
        let categories = loader
            .load_all()
            .expect("production mode should skip bad category files");

        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].id, "movies");
    }

    #[test]
    fn loader_rejects_duplicate_category_ids() {
        let dir = temp_category_dir("duplicate-id");
        fs::write(
            dir.join("first.json"),
            valid_category_json("movies", "Movies"),
        )
        .expect("first category should be written");
        fs::write(
            dir.join("second.json"),
            valid_category_json("movies", "More Movies"),
        )
        .expect("second category should be written");

        let loader = CategoryPackLoader::strict(&dir);
        let err = loader
            .load_all()
            .expect_err("duplicate ids should be rejected");

        assert!(err.contains("duplicate category id 'movies'"));
    }
}
