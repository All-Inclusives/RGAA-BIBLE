//! Portable RGAA data, search, filters, and editorial helpers for AccessiGuide.
//!
//! The bundled dataset is the validated RGAA 4.1.2 corpus. Use [`A11yCore::from_bundled`]
//! for the embedded version or [`A11yCore::from_json_str`] to load an updated dataset.

use std::{
    collections::{BTreeMap, HashSet},
    fmt, fs,
    path::Path,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

mod updates;

pub use updates::{
    InstalledUpdate, UpdateError, UpdateManifest, install_signed_update, install_update,
    latest_installed_update, load_from_update_store, load_or_bundled, load_signed_or_bundled,
};

const BUNDLED_DATA: &str = include_str!("../../../data/rgaa-4.1.2.json");
const SUPPORTED_DATASET_SCHEMA_VERSIONS: &[&str] = &["1.0.0", "1.1.0"];

#[derive(Debug)]
pub enum CoreError {
    Io(std::io::Error),
    InvalidJson(serde_json::Error),
    InvalidDataset(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "unable to read dataset: {error}"),
            Self::InvalidJson(error) => write!(formatter, "invalid dataset JSON: {error}"),
            Self::InvalidDataset(message) => write!(formatter, "invalid dataset: {message}"),
        }
    }
}

impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidJson(error) => Some(error),
            Self::InvalidDataset(_) => None,
        }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidJson(error)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    pub schema_version: String,
    pub source: DatasetSource,
    pub wcag_version: f64,
    pub topics: Vec<Topic>,
    pub glossary: Vec<GlossaryEntry>,
    pub methodologies: BTreeMap<String, String>,
    pub counts: DatasetCounts,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DatasetSource {
    pub repository: String,
    pub commit: String,
    pub rgaa_version: String,
    pub license: String,
    pub files: Vec<SourceFile>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DatasetCounts {
    pub topics: usize,
    pub criteria: usize,
    pub tests: usize,
    pub glossary_entries: usize,
    pub methodologies: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Topic {
    pub id: String,
    pub title: String,
    pub criteria: Vec<Criterion>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Criterion {
    pub id: String,
    pub official: OfficialCriterion,
    pub editorial: EditorialContent,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OfficialCriterion {
    pub number: u32,
    pub title: String,
    pub tests: BTreeMap<String, Value>,
    pub references: Vec<BTreeMap<String, Value>>,
    #[serde(flatten)]
    pub additional_fields: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EditorialContent {
    pub fr: LocalizedEditorialContent,
    pub tags: EditorialTags,
    pub examples: Vec<CodeExample>,
    #[serde(default, rename = "pedagogicalTests")]
    pub pedagogical_tests: Vec<PedagogicalTestContent>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PedagogicalTestContent {
    pub test_id: String,
    pub status: PedagogicalContentStatus,
    pub plain_language_draft: String,
    pub user_benefit: String,
    pub cases: Vec<PedagogicalCase>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PedagogicalContentStatus {
    Draft,
    InReview,
    Validated,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PedagogicalCase {
    pub id: String,
    pub title: String,
    pub context: String,
    pub explanation: String,
    #[serde(default)]
    pub visual_effect: Option<String>,
    pub resources: Vec<PedagogicalResource>,
    pub code: PedagogicalCode,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PedagogicalResource {
    pub id: String,
    pub file: String,
    pub alt_text: Option<String>,
    pub source: String,
    pub license: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PedagogicalCode {
    pub language: String,
    pub snippet: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LocalizedEditorialContent {
    pub explanation: Option<String>,
    #[serde(rename = "userImpact")]
    pub user_impact: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EditorialTags {
    pub components: Vec<String>,
    #[serde(rename = "disabilitySituations")]
    pub disability_situations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CodeExample {
    pub language: String,
    pub bad: String,
    pub good: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GlossaryEntry {
    pub title: String,
    #[serde(rename = "bodyHtml")]
    pub body_html: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArticleQuery {
    /// Optional free-text query. Matching ignores case and diacritics.
    pub text: Option<String>,
    /// Topic number or topic title.
    pub topic_id: Option<String>,
    /// Exact editorial component tag, compared case- and diacritic-insensitively.
    pub component: Option<String>,
    /// Exact editorial disability-situation tag, compared case- and diacritic-insensitively.
    pub disability_situation: Option<String>,
    /// Maximum number of matches; `None` returns all matching criteria.
    pub limit: Option<usize>,
}

/// Describes compatibility of the loaded corpus with caller-expected versions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataVersionCheck {
    pub bundled_schema_version: String,
    pub bundled_rgaa_version: String,
    pub bundled_source_commit: String,
    pub schema_matches: bool,
    pub rgaa_version_matches: bool,
}

#[derive(Clone, Debug)]
pub struct A11yCore {
    dataset: Dataset,
    search_index: Vec<Vec<String>>,
}

impl A11yCore {
    /// Loads and validates the RGAA corpus embedded at compile time.
    pub fn from_bundled() -> Result<Self, CoreError> {
        Self::from_json_str(BUNDLED_DATA)
    }

    /// Parses and validates a dataset encoded with the AccessiGuide schema.
    pub fn from_json_str(json: &str) -> Result<Self, CoreError> {
        let dataset: Dataset = serde_json::from_str(json)?;
        Self::new(dataset)
    }

    /// Reads, parses, and validates a dataset from a local file.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, CoreError> {
        let json = fs::read_to_string(path)?;
        Self::from_json_str(&json)
    }

    /// Constructs the search index after checking the dataset invariants.
    pub fn new(dataset: Dataset) -> Result<Self, CoreError> {
        validate_dataset(&dataset)?;
        let search_index = dataset
            .topics
            .iter()
            .map(|topic| {
                topic
                    .criteria
                    .iter()
                    .map(|criterion| normalized_criterion_text(topic, criterion))
                    .collect()
            })
            .collect();
        Ok(Self {
            dataset,
            search_index,
        })
    }

    /// Returns the validated dataset, including official and editorial content.
    pub fn dataset(&self) -> &Dataset {
        &self.dataset
    }

    /// Searches criteria and applies optional topic and editorial-tag filters.
    ///
    /// Results rank exact references first, then reference prefixes and title
    /// prefixes, while preserving source order for ties. An empty query returns
    /// criteria matching the selected filters.
    pub fn query_articles(&self, query: &ArticleQuery) -> Vec<&Criterion> {
        let normalized_text = query
            .text
            .as_deref()
            .map(normalize)
            .filter(|text| !text.is_empty());
        let tokens: Vec<&str> = normalized_text
            .as_deref()
            .unwrap_or_default()
            .split_whitespace()
            .collect();

        let mut results = Vec::new();
        for (topic_index, topic) in self.dataset.topics.iter().enumerate() {
            if query.topic_id.as_deref().is_some_and(|topic_id| {
                normalize(topic_id) != normalize(&topic.id)
                    && normalize(topic_id) != normalize(&topic.title)
            }) {
                continue;
            }

            for (criterion_index, criterion) in topic.criteria.iter().enumerate() {
                if !matches_tag(
                    query.component.as_deref(),
                    &criterion.editorial.tags.components,
                ) || !matches_tag(
                    query.disability_situation.as_deref(),
                    &criterion.editorial.tags.disability_situations,
                ) {
                    continue;
                }

                let search_text = &self.search_index[topic_index][criterion_index];
                if tokens.iter().all(|token| search_text.contains(token)) {
                    let rank = search_rank(
                        normalized_text.as_deref(),
                        search_text,
                        &criterion.id,
                        &criterion.official.title,
                    );
                    results.push((rank, results.len(), criterion));
                }
            }
        }

        results.sort_by_key(|(rank, source_order, _)| (*rank, *source_order));
        results
            .into_iter()
            .take(query.limit.unwrap_or(usize::MAX))
            .map(|(_, _, criterion)| criterion)
            .collect()
    }

    /// Looks up a criterion by reference, accepting punctuation and whitespace
    /// variations such as `1.1` and `1-1`.
    pub fn get_criterion(&self, reference: &str) -> Option<&Criterion> {
        let normalized_reference = normalize_reference(reference);
        self.dataset
            .topics
            .iter()
            .flat_map(|topic| &topic.criteria)
            .find(|criterion| normalize_reference(&criterion.id) == normalized_reference)
    }

    /// Returns criteria tagged with the given disability situation.
    pub fn filter_by_disability(&self, situation: &str) -> Vec<&Criterion> {
        self.query_articles(&ArticleQuery {
            disability_situation: Some(situation.to_owned()),
            ..ArticleQuery::default()
        })
    }

    /// Returns the editorial code examples for a criterion reference.
    pub fn get_code_examples(&self, reference: &str) -> Option<&[CodeExample]> {
        self.get_criterion(reference)
            .map(|criterion| criterion.editorial.examples.as_slice())
    }

    /// Reports whether the loaded corpus matches the expected schema and RGAA
    /// versions, along with the source commit actually loaded.
    pub fn check_data_version(
        &self,
        expected_schema_version: &str,
        expected_rgaa_version: &str,
    ) -> DataVersionCheck {
        DataVersionCheck {
            bundled_schema_version: self.dataset.schema_version.clone(),
            bundled_rgaa_version: self.dataset.source.rgaa_version.clone(),
            bundled_source_commit: self.dataset.source.commit.clone(),
            schema_matches: self.dataset.schema_version == expected_schema_version,
            rgaa_version_matches: self.dataset.source.rgaa_version == expected_rgaa_version,
        }
    }
}

fn validate_dataset(dataset: &Dataset) -> Result<(), CoreError> {
    let invalid = |message: String| CoreError::InvalidDataset(message);
    if !SUPPORTED_DATASET_SCHEMA_VERSIONS.contains(&dataset.schema_version.as_str()) {
        return Err(invalid(format!(
            "unsupported dataset schema version {}",
            dataset.schema_version
        )));
    }
    if dataset.topics.is_empty() {
        return Err(invalid("no topics found".to_owned()));
    }
    if dataset.source.commit.len() != 40
        || !dataset
            .source
            .commit
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(invalid(
            "source commit must be a 40-character Git hash".to_owned(),
        ));
    }

    let mut topic_ids = HashSet::new();
    let mut criterion_ids = HashSet::new();
    let mut test_ids = HashSet::new();
    for topic in &dataset.topics {
        if topic.id.trim().is_empty() || topic.title.trim().is_empty() {
            return Err(invalid("topic id and title must not be empty".to_owned()));
        }
        if !topic_ids.insert(topic.id.clone()) {
            return Err(invalid(format!("duplicate topic id: {}", topic.id)));
        }
        for criterion in &topic.criteria {
            let expected_id = format!("{}.{}", topic.id, criterion.official.number);
            if criterion.id != expected_id {
                return Err(invalid(format!(
                    "criterion id {} does not match expected id {expected_id}",
                    criterion.id
                )));
            }
            if !criterion_ids.insert(criterion.id.clone()) {
                return Err(invalid(format!("duplicate criterion id: {}", criterion.id)));
            }
            let mut pedagogy_ids = HashSet::new();
            for pedagogy in &criterion.editorial.pedagogical_tests {
                let test_number = pedagogy
                    .test_id
                    .strip_prefix(&format!("{}.", criterion.id))
                    .unwrap_or("");
                if !criterion.official.tests.contains_key(test_number)
                    || !pedagogy_ids.insert(pedagogy.test_id.as_str())
                {
                    return Err(invalid(format!(
                        "pedagogical content references an unknown or duplicate test: {}",
                        pedagogy.test_id
                    )));
                }
                if pedagogy.plain_language_draft.trim().is_empty()
                    || pedagogy.user_benefit.trim().is_empty()
                    || pedagogy.cases.is_empty()
                {
                    return Err(invalid(format!(
                        "pedagogical content for {} must include an explanation, user benefit, and cases",
                        pedagogy.test_id
                    )));
                }
                let mut case_ids = HashSet::new();
                for case in &pedagogy.cases {
                    if case.id.trim().is_empty()
                        || !case_ids.insert(case.id.as_str())
                        || case.title.trim().is_empty()
                        || case.context.trim().is_empty()
                        || case.explanation.trim().is_empty()
                        || case.resources.is_empty()
                        || case.code.language.trim().is_empty()
                        || case.code.snippet.trim().is_empty()
                        || case
                            .visual_effect
                            .as_deref()
                            .is_some_and(|effect| effect != "blur")
                    {
                        return Err(invalid(format!(
                            "pedagogical case {} in {} is incomplete or invalid",
                            case.id, pedagogy.test_id
                        )));
                    }
                    let mut resource_ids = HashSet::new();
                    if case.resources.iter().any(|resource| {
                        resource.id.trim().is_empty()
                            || !resource_ids.insert(resource.id.as_str())
                            || resource.file.trim().is_empty()
                            || !resource.file.chars().all(|character| {
                                character.is_ascii_alphanumeric() || "._-".contains(character)
                            })
                            || !resource
                                .file
                                .chars()
                                .next()
                                .is_some_and(|character| character.is_ascii_alphanumeric())
                            || resource.source.trim().is_empty()
                            || resource.license.trim().is_empty()
                            || !case.code.snippet.contains(&resource.file)
                    }) {
                        return Err(invalid(format!(
                            "pedagogical resources in {} must have unique ids and complete provenance",
                            pedagogy.test_id
                        )));
                    }
                }
            }
            for test_number in criterion.official.tests.keys() {
                let test_id = format!("{}.{}", criterion.id, test_number);
                if !test_ids.insert(test_id.clone()) {
                    return Err(invalid(format!("duplicate test id: {test_id}")));
                }
            }
        }
    }

    let methodology_ids: HashSet<&str> = dataset.methodologies.keys().map(String::as_str).collect();
    if let Some(missing) = test_ids
        .iter()
        .find(|test_id| !methodology_ids.contains(test_id.as_str()))
    {
        return Err(invalid(format!("missing methodology for test {missing}")));
    }
    if let Some(extra) = methodology_ids
        .iter()
        .find(|test_id| !test_ids.contains(**test_id))
    {
        return Err(invalid(format!(
            "methodology has no matching test: {extra}"
        )));
    }

    if dataset.counts.topics != dataset.topics.len()
        || dataset.counts.criteria != criterion_ids.len()
        || dataset.counts.tests != test_ids.len()
        || dataset.counts.glossary_entries != dataset.glossary.len()
        || dataset.counts.methodologies != dataset.methodologies.len()
    {
        return Err(invalid(
            "dataset counts do not match its contents".to_owned(),
        ));
    }
    Ok(())
}

fn normalize(value: &str) -> String {
    value
        .nfkd()
        .filter(|character| !is_combining_mark(*character))
        .flat_map(char::to_lowercase)
        .collect()
}

fn normalize_reference(reference: &str) -> String {
    reference
        .trim()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn matches_tag(query: Option<&str>, tags: &[String]) -> bool {
    query.is_none_or(|query| {
        let query = normalize(query.trim());
        !query.is_empty() && tags.iter().any(|tag| normalize(tag) == query)
    })
}

fn normalized_criterion_text(topic: &Topic, criterion: &Criterion) -> String {
    let mut text = vec![
        topic.title.clone(),
        criterion.id.clone(),
        criterion.official.title.clone(),
    ];
    for value in criterion.official.tests.values() {
        append_json_strings(value, &mut text);
    }
    for value in criterion.official.additional_fields.values() {
        append_json_strings(value, &mut text);
    }
    for reference in &criterion.official.references {
        for value in reference.values() {
            append_json_strings(value, &mut text);
        }
    }
    if let Some(explanation) = &criterion.editorial.fr.explanation {
        text.push(explanation.clone());
    }
    if let Some(impact) = &criterion.editorial.fr.user_impact {
        text.push(impact.clone());
    }
    text.extend(criterion.editorial.tags.components.iter().cloned());
    text.extend(
        criterion
            .editorial
            .tags
            .disability_situations
            .iter()
            .cloned(),
    );
    for example in &criterion.editorial.examples {
        text.push(example.language.clone());
        text.push(example.bad.clone());
        text.push(example.good.clone());
    }
    normalize(&text.join(" "))
}

fn append_json_strings(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::String(value) => output.push(value.clone()),
        Value::Array(values) => {
            for value in values {
                append_json_strings(value, output);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                append_json_strings(value, output);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn search_rank(query: Option<&str>, normalized_text: &str, criterion_id: &str, title: &str) -> u8 {
    let Some(query) = query else {
        return 0;
    };
    let query = query.trim();
    if normalize_reference(criterion_id) == normalize_reference(query) {
        0
    } else if normalize_reference(criterion_id).starts_with(&normalize_reference(query)) {
        1
    } else if normalize(title).starts_with(&normalize(query)) {
        2
    } else if normalized_text.contains(&normalize(query)) {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_bundled_corpus_and_checks_version() {
        let core = A11yCore::from_bundled().expect("bundled corpus should be valid");
        assert_eq!(core.dataset().counts.criteria, 106);
        assert_eq!(core.dataset().counts.tests, 258);
        assert_eq!(core.dataset().counts.glossary_entries, 119);
        assert_eq!(core.dataset().counts.methodologies, 258);

        let version = core.check_data_version("1.1.0", "4.1.2");
        assert!(version.schema_matches);
        assert!(version.rgaa_version_matches);
        assert_eq!(
            version.bundled_source_commit,
            "ca4019f95073b6cbd2482a16e9f12b52d8de678d"
        );
    }

    #[test]
    fn finds_criterion_by_reference_and_ranks_exact_match_first() {
        let core = A11yCore::from_bundled().expect("bundled corpus should be valid");
        let criterion = core.get_criterion(" 1.1 ").expect("criterion 1.1");
        assert_eq!(criterion.id, "1.1");

        let results = core.query_articles(&ArticleQuery {
            text: Some("1.1".to_owned()),
            ..ArticleQuery::default()
        });
        assert_eq!(results[0].id, "1.1");
    }

    #[test]
    fn searches_official_text_without_accents_or_case_sensitivity() {
        let core = A11yCore::from_bundled().expect("bundled corpus should be valid");
        let results = core.query_articles(&ArticleQuery {
            text: Some("ALTERNATIVE TEXTUELLE".to_owned()),
            ..ArticleQuery::default()
        });
        assert!(results.iter().any(|criterion| criterion.id == "1.1"));

        let accented_results = core.query_articles(&ArticleQuery {
            text: Some("méthodologie".to_owned()),
            ..ArticleQuery::default()
        });
        let unaccented_results = core.query_articles(&ArticleQuery {
            text: Some("methodologie".to_owned()),
            ..ArticleQuery::default()
        });
        assert_eq!(
            accented_results
                .iter()
                .map(|criterion| criterion.id.as_str())
                .collect::<Vec<_>>(),
            unaccented_results
                .iter()
                .map(|criterion| criterion.id.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn supports_filters_limits_and_empty_results() {
        let core = A11yCore::from_bundled().expect("bundled corpus should be valid");
        assert!(
            core.query_articles(&ArticleQuery {
                component: Some("formulaire".to_owned()),
                ..ArticleQuery::default()
            })
            .is_empty()
        );
        assert!(
            core.query_articles(&ArticleQuery {
                text: Some("this cannot match an RGAA criterion".to_owned()),
                ..ArticleQuery::default()
            })
            .is_empty()
        );
        assert_eq!(
            core.query_articles(&ArticleQuery {
                limit: Some(3),
                ..ArticleQuery::default()
            })
            .len(),
            3
        );
        assert!(
            core.query_articles(&ArticleQuery {
                topic_id: Some("Images".to_owned()),
                limit: Some(1),
                ..ArticleQuery::default()
            })
            .iter()
            .all(|criterion| criterion.id.starts_with("1."))
        );
    }

    #[test]
    fn filters_editorial_tags_and_returns_examples() {
        let mut dataset: Dataset =
            serde_json::from_str(BUNDLED_DATA).expect("bundled JSON should parse");
        let criterion = &mut dataset.topics[0].criteria[0];
        criterion
            .editorial
            .tags
            .components
            .push("Formulaire".to_owned());
        criterion
            .editorial
            .tags
            .disability_situations
            .push("Déficience visuelle".to_owned());
        criterion.editorial.examples.push(CodeExample {
            language: "html".to_owned(),
            bad: "<button><img src=\"cart.svg\"></button>".to_owned(),
            good:
                "<button aria-label=\"Valider le panier\"><img src=\"cart.svg\" alt=\"\"></button>"
                    .to_owned(),
        });
        let core = A11yCore::new(dataset).expect("editorial fixture should be valid");

        let filtered = core.query_articles(&ArticleQuery {
            component: Some("formulaire".to_owned()),
            disability_situation: Some("deficience visuelle".to_owned()),
            ..ArticleQuery::default()
        });
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "1.1");
        assert_eq!(core.filter_by_disability("DÉFICIENCE VISUELLE").len(), 1);
        assert_eq!(core.get_code_examples("1.1").unwrap().len(), 1);
    }

    #[test]
    fn rejects_malformed_json_and_inconsistent_test_data() {
        assert!(matches!(
            A11yCore::from_json_str("{"),
            Err(CoreError::InvalidJson(_))
        ));

        let mut dataset: Dataset =
            serde_json::from_str(BUNDLED_DATA).expect("bundled JSON should parse");
        dataset.methodologies.remove("1.1.1");
        assert!(matches!(
            A11yCore::new(dataset),
            Err(CoreError::InvalidDataset(_))
        ));
    }

    #[test]
    fn returns_empty_examples_for_valid_editorial_criterion() {
        let core = A11yCore::from_bundled().expect("bundled corpus should be valid");
        assert_eq!(
            core.get_code_examples("1.1").expect("criterion exists"),
            &[]
        );
        assert!(core.get_code_examples("99.99").is_none());
    }

    #[test]
    fn loads_the_pedagogical_pilot_and_distinguishes_alternative_states() {
        let core = A11yCore::from_bundled().expect("bundled corpus should be valid");
        let criterion = core.get_criterion("1.1").expect("criterion 1.1");
        let pilot = criterion
            .editorial
            .pedagogical_tests
            .iter()
            .find(|test| test.test_id == "1.1.1")
            .expect("pedagogical pilot for test 1.1.1");

        assert_eq!(pilot.cases.len(), 4);
        assert_eq!(pilot.cases[0].resources[0].alt_text.as_deref(), Some(""));
        assert_eq!(pilot.cases[1].resources[0].alt_text, None);
        assert_eq!(
            pilot.cases[2].resources[0].alt_text.as_deref(),
            Some("image")
        );
        assert!(
            pilot.cases[3].resources[0]
                .alt_text
                .as_deref()
                .is_some_and(|alternative| alternative.contains("tasse verte"))
        );
    }

    #[test]
    fn loads_legacy_dataset_without_pedagogical_fields() {
        let mut legacy: Value =
            serde_json::from_str(BUNDLED_DATA).expect("bundled JSON should parse");
        legacy["schemaVersion"] = Value::String("1.0.0".to_owned());
        for topic in legacy["topics"]
            .as_array_mut()
            .expect("topics should be an array")
        {
            for criterion in topic["criteria"]
                .as_array_mut()
                .expect("criteria should be an array")
            {
                criterion["editorial"]
                    .as_object_mut()
                    .expect("editorial should be an object")
                    .remove("pedagogicalTests");
            }
        }

        let core = A11yCore::from_json_str(
            &serde_json::to_string(&legacy).expect("legacy dataset should serialize"),
        )
        .expect("legacy corpus should remain compatible");
        assert_eq!(
            core.get_criterion("1.1")
                .expect("criterion 1.1")
                .editorial
                .pedagogical_tests,
            []
        );
    }
}
