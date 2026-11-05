mod update_client;

use std::{collections::BTreeMap, path::PathBuf, sync::RwLock};

use a11y_core::{A11yCore, ArticleQuery, Criterion, DatasetCounts, Topic, load_signed_or_bundled};
use tauri::{Manager, State};
use update_client::UpdateCheckResult;

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    topic_id: String,
    topic_title: String,
    criterion: Criterion,
}

struct AppState {
    core: RwLock<A11yCore>,
    update_store: PathBuf,
    trusted_public_key: Option<[u8; 32]>,
    update_lock: tokio::sync::Mutex<()>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DatasetInfo {
    methodologies: BTreeMap<String, String>,
    counts: DatasetCounts,
}

#[tauri::command]
fn list_topics(state: State<'_, AppState>) -> Result<Vec<Topic>, String> {
    let core = state
        .core
        .read()
        .map_err(|error| format!("Le cœur RGAA est indisponible : {error}"))?;
    Ok(core.dataset().topics.clone())
}

#[tauri::command]
fn get_dataset_info(state: State<'_, AppState>) -> Result<DatasetInfo, String> {
    let core = state
        .core
        .read()
        .map_err(|error| format!("Le cœur RGAA est indisponible : {error}"))?;
    Ok(DatasetInfo {
        methodologies: core.dataset().methodologies.clone(),
        counts: core.dataset().counts.clone(),
    })
}

#[tauri::command]
fn query_articles(
    state: State<'_, AppState>,
    text: Option<String>,
    topic_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>, String> {
    let core = state
        .core
        .read()
        .map_err(|error| format!("Le cœur RGAA est indisponible : {error}"))?;
    let query = ArticleQuery {
        text,
        topic_id,
        component: None,
        disability_situation: None,
        limit,
    };

    Ok(core
        .query_articles(&query)
        .into_iter()
        .filter_map(|criterion| {
            core.dataset()
                .topics
                .iter()
                .find(|topic| topic.criteria.iter().any(|entry| entry.id == criterion.id))
                .map(|topic| SearchResult {
                    topic_id: topic.id.clone(),
                    topic_title: topic.title.clone(),
                    criterion: criterion.clone(),
                })
        })
        .collect())
}

#[tauri::command]
fn get_criterion(
    state: State<'_, AppState>,
    reference: String,
) -> Result<Option<SearchResult>, String> {
    let core = state
        .core
        .read()
        .map_err(|error| format!("Le cœur RGAA est indisponible : {error}"))?;
    let Some(criterion) = core.get_criterion(&reference) else {
        return Ok(None);
    };
    let topic = core
        .dataset()
        .topics
        .iter()
        .find(|topic| topic.criteria.iter().any(|entry| entry.id == criterion.id));
    Ok(topic.map(|topic| SearchResult {
        topic_id: topic.id.clone(),
        topic_title: topic.title.clone(),
        criterion: criterion.clone(),
    }))
}

#[tauri::command]
async fn check_for_updates(state: State<'_, AppState>) -> Result<UpdateCheckResult, String> {
    let _update_guard = state.update_lock.lock().await;
    let trusted_public_key = state.trusted_public_key.ok_or_else(|| {
        "Les mises à jour signées ne sont pas configurées : la clé publique de confiance est absente."
            .to_owned()
    })?;
    let result = update_client::check_and_install(&state.update_store, &trusted_public_key).await?;

    if result.updated {
        let updated_core = load_signed_or_bundled(&state.update_store, &trusted_public_key)
            .map_err(|error| {
                format!("La mise à jour installée n’a pas pu être chargée : {error}")
            })?;
        *state.core.write().map_err(|error| {
            format!("Le cœur RGAA ne peut pas activer la mise à jour : {error}")
        })? = updated_core;
    }

    Ok(result)
}

fn trusted_update_public_key() -> Result<Option<[u8; 32]>, String> {
    let Some(encoded) = option_env!("A11Y_UPDATE_PUBLIC_KEY") else {
        return Ok(None);
    };
    if encoded.len() != 64 || !encoded.is_ascii() {
        return Err("A11Y_UPDATE_PUBLIC_KEY doit contenir 64 caractères hexadécimaux.".to_owned());
    }

    let mut key = [0; 32];
    for (index, pair) in encoded.as_bytes().chunks_exact(2).enumerate() {
        key[index] = (hex_digit(pair[0])? << 4) | hex_digit(pair[1])?;
    }
    Ok(Some(key))
}

fn hex_digit(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err("A11Y_UPDATE_PUBLIC_KEY doit être en hexadécimal.".to_owned()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let update_store = app.path().app_data_dir()?.join("rgaa-updates");
            let trusted_public_key = trusted_update_public_key().map_err(std::io::Error::other)?;
            let core = match trusted_public_key {
                Some(key) => {
                    load_signed_or_bundled(&update_store, &key).map_err(std::io::Error::other)?
                }
                None => A11yCore::from_bundled()?,
            };
            app.manage(AppState {
                core: RwLock::new(core),
                update_store,
                trusted_public_key,
                update_lock: tokio::sync::Mutex::new(()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_topics,
            get_dataset_info,
            query_articles,
            get_criterion,
            check_for_updates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
