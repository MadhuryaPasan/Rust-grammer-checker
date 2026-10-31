use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub updated_at: u128,
}

impl Document {
    pub fn new(title: String) -> Self {
        Self {
            id: new_id(),
            title,
            content: String::new(),
            updated_at: now(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
    pub english_variant: String,
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    pub temperature: String,
    pub thinking_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            english_variant: "US English".to_owned(),
            provider: "Ollama".to_owned(),
            model: "qwen2.5:3b".to_owned(),
            endpoint: "http://localhost:11434".to_owned(),
            temperature: "0.1".to_owned(),
            thinking_enabled: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Store {
    pub documents: Vec<Document>,
    pub settings: Settings,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            documents: vec![Document::new("Untitled document".to_owned())],
            settings: Settings::default(),
        }
    }
}

#[derive(Clone)]
pub struct LoadedStore {
    pub store: Store,
    pub error: Option<String>,
}

pub fn load_store() -> LoadedStore {
    let path = match store_path() {
        Ok(path) => path,
        Err(error) => {
            return LoadedStore {
                store: Store::default(),
                error: Some(error),
            }
        }
    };

    match fs::read(&path) {
        Ok(bytes) => match serde_json::from_slice::<Store>(&bytes) {
            Ok(mut store) => {
                store
                    .documents
                    .sort_by_key(|document| std::cmp::Reverse(document.updated_at));
                LoadedStore { store, error: None }
            }
            Err(error) => LoadedStore {
                store: Store::default(),
                error: Some(format!(
                    "Could not read saved documents at {}: {error}",
                    path.display()
                )),
            },
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let store = Store::default();
            let save_error = save_store_to_path(&path, &store).err();
            LoadedStore {
                store,
                error: save_error,
            }
        }
        Err(error) => LoadedStore {
            store: Store::default(),
            error: Some(format!(
                "Could not open saved documents at {}: {error}",
                path.display()
            )),
        },
    }
}

pub fn save_store(store: &Store) -> Result<(), String> {
    let path = store_path()?;
    save_store_to_path(&path, store)
}

fn store_path() -> Result<PathBuf, String> {
    let data_dir = if let Some(path) = env::var_os("APPDATA") {
        PathBuf::from(path).join("Grammar Check")
    } else if let Some(path) = env::var_os("LOCALAPPDATA") {
        PathBuf::from(path).join("Grammar Check")
    } else if let Some(path) = env::var_os("HOME") {
        PathBuf::from(path)
            .join(".local")
            .join("share")
            .join("Grammar Check")
    } else {
        return Err("Could not locate a user data folder for saved documents.".to_owned());
    };

    Ok(data_dir.join("documents.json"))
}

fn save_store_to_path(path: &Path, store: &Store) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| {
        format!(
            "Could not determine the folder for saved documents: {}",
            path.display()
        )
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;

    let bytes = serde_json::to_vec_pretty(store)
        .map_err(|error| format!("Could not encode saved documents: {error}"))?;
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, bytes).map_err(|error| {
        format!(
            "Could not write saved documents to {}: {error}",
            temporary_path.display()
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|error| {
        let _ = fs::remove_file(&temporary_path);
        format!(
            "Could not replace saved documents at {}: {error}",
            path.display()
        )
    })
}

fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn timestamp() -> u128 {
    now()
}

fn new_id() -> String {
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("document-{}-{sequence}", now())
}

#[cfg(test)]
mod tests {
    use super::{save_store_to_path, Document, Settings, Store};
    use std::fs;

    #[test]
    fn round_trips_documents_and_settings() {
        let path = std::env::temp_dir().join(format!(
            "grammar-check-storage-test-{}-{}.json",
            std::process::id(),
            super::now()
        ));
        let mut store = Store {
            documents: vec![Document {
                id: "doc-1".to_owned(),
                title: "Test".to_owned(),
                content: "A saved draft.".to_owned(),
                updated_at: 123,
            }],
            settings: Settings {
                english_variant: "UK English".to_owned(),
                provider: "Ollama".to_owned(),
                model: "qwen3:1.7b".to_owned(),
                ..Settings::default()
            },
        };

        save_store_to_path(&path, &store).expect("save should succeed");
        store.documents[0].content = "Updated draft.".to_owned();
        save_store_to_path(&path, &store).expect("existing save file should be replaced");
        let restored: Store =
            serde_json::from_slice(&fs::read(&path).expect("saved file should exist"))
                .expect("saved JSON should deserialize");
        fs::remove_file(&path).expect("test file should be removed");

        assert_eq!(restored, store);
    }

    #[test]
    fn default_store_has_an_editable_document() {
        let store = Store::default();
        assert_eq!(store.documents.len(), 1);
        assert!(!store.documents[0].id.is_empty());
        assert_eq!(store.documents[0].title, "Untitled document");
        assert_eq!(store.settings.english_variant, "US English");
    }
}
