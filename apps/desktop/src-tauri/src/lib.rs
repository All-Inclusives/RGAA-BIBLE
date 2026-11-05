use a11y_core::{A11yCore, ArticleQuery, Criterion, Topic};
use tauri::{Manager, State};

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    topic_id: String,
    topic_title: String,
    criterion: Criterion,
}

#[tauri::command]
fn list_topics(core: State<'_, A11yCore>) -> Vec<Topic> {
    core.dataset().topics.clone()
}

#[tauri::command]
fn query_articles(
    core: State<'_, A11yCore>,
    text: Option<String>,
    topic_id: Option<String>,
    limit: Option<usize>,
) -> Vec<SearchResult> {
    let query = ArticleQuery {
        text,
        topic_id,
        component: None,
        disability_situation: None,
        limit,
    };

    core.query_articles(&query)
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
        .collect()
}

#[tauri::command]
fn get_criterion(core: State<'_, A11yCore>, reference: String) -> Option<SearchResult> {
    let criterion = core.get_criterion(&reference)?;
    let topic = core
        .dataset()
        .topics
        .iter()
        .find(|topic| topic.criteria.iter().any(|entry| entry.id == criterion.id))?;
    Some(SearchResult {
        topic_id: topic.id.clone(),
        topic_title: topic.title.clone(),
        criterion: criterion.clone(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let core = A11yCore::from_bundled()?;
            app.manage(core);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_topics,
            query_articles,
            get_criterion
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
