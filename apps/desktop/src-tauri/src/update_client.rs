use std::{path::Path, time::Duration};

use a11y_core::{UpdateManifest, install_signed_update, latest_installed_update};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

const RELEASES_URL: &str =
    "https://api.github.com/repos/All-Inclusives/RGAA-BIBLE/releases?per_page=20";
const DOWNLOAD_BASE: &str = "https://github.com/All-Inclusives/RGAA-BIBLE/releases/download";
const MAX_RELEASES_BYTES: usize = 1_000_000;
const MAX_MANIFEST_BYTES: usize = 16_384;
const MAX_PACKAGE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub updated: bool,
    pub sequence: Option<u64>,
    pub rgaa_version: Option<String>,
}

pub async fn check_and_install(
    store_dir: &Path,
    trusted_public_key: &[u8; 32],
) -> Result<UpdateCheckResult, String> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("AccessiGuide/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            let host = attempt.url().host_str().unwrap_or_default();
            if attempt.previous().len() >= 5
                || !(host == "github.com"
                    || host == "api.github.com"
                    || host == "objects.githubusercontent.com"
                    || host == "release-assets.githubusercontent.com"
                    || host == "github-releases.githubusercontent.com")
            {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|error| {
            format!("Impossible de préparer la connexion aux mises à jour : {error}")
        })?;

    let response = client
        .get(RELEASES_URL)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| format!("Impossible de consulter les versions RGAA : {error}"))?;
    let releases: Vec<GitHubRelease> =
        serde_json::from_slice(&read_limited(response, MAX_RELEASES_BYTES).await?)
            .map_err(|error| format!("Réponse des versions RGAA invalide : {error}"))?;

    let latest = latest_release(releases);

    let Some((sequence, release)) = latest else {
        return Ok(UpdateCheckResult {
            updated: false,
            sequence: None,
            rgaa_version: None,
        });
    };

    if let Some(installed) =
        latest_installed_update(store_dir).map_err(|error| error.to_string())?
    {
        if sequence < installed.sequence {
            return Err(format!(
                "La dernière publication ({sequence}) est antérieure à la mise à jour installée ({}).",
                installed.sequence
            ));
        }
        if sequence == installed.sequence {
            return Ok(UpdateCheckResult {
                updated: false,
                sequence: Some(sequence),
                rgaa_version: Some(installed.rgaa_version),
            });
        }
    }

    let tag = format!("rgaa-data-{sequence}");
    let manifest_url = download_url(&release, &tag, "manifest.json")?;
    let manifest_bytes = read_limited(
        client
            .get(manifest_url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| format!("Impossible de télécharger le manifeste RGAA : {error}"))?,
        MAX_MANIFEST_BYTES,
    )
    .await?;
    let manifest: UpdateManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("Manifeste de mise à jour invalide : {error}"))?;
    if manifest.sequence != sequence {
        return Err("Le numéro de séquence ne correspond pas à la Release.".to_owned());
    }
    if manifest.package_bytes > MAX_PACKAGE_BYTES as u64 {
        return Err("Le paquet RGAA publié dépasse la taille maximale autorisée.".to_owned());
    }
    manifest
        .verify_signature(trusted_public_key)
        .map_err(|error| format!("Signature du manifeste refusée : {error}"))?;
    if !manifest.package_file.starts_with("rgaa-")
        || !manifest.package_file.ends_with(".json")
        || manifest.package_file.contains('/')
        || manifest.package_file.contains('\\')
    {
        return Err("Le nom du paquet RGAA publié est invalide.".to_owned());
    }

    let package_url = download_url(&release, &tag, &manifest.package_file)?;
    let package_bytes = read_limited(
        client
            .get(package_url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| format!("Impossible de télécharger le corpus RGAA : {error}"))?,
        MAX_PACKAGE_BYTES,
    )
    .await?;
    let installed = install_signed_update(
        store_dir,
        &manifest_bytes,
        &package_bytes,
        trusted_public_key,
    )
    .map_err(|error| format!("La mise à jour RGAA a été rejetée : {error}"))?;

    Ok(UpdateCheckResult {
        updated: true,
        sequence: Some(installed.sequence),
        rgaa_version: Some(installed.rgaa_version),
    })
}

fn latest_release(releases: Vec<GitHubRelease>) -> Option<(u64, GitHubRelease)> {
    releases
        .into_iter()
        .filter(|release| !release.draft && !release.prerelease)
        .filter_map(|release| {
            let sequence = release
                .tag_name
                .strip_prefix("rgaa-data-")?
                .parse::<u64>()
                .ok()?;
            (sequence > 0).then_some((sequence, release))
        })
        .max_by_key(|(sequence, _)| *sequence)
}

fn download_url(release: &GitHubRelease, tag: &str, filename: &str) -> Result<String, String> {
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == filename)
        .ok_or_else(|| format!("La Release {tag} ne contient pas {filename}."))?;
    let expected_url = format!("{DOWNLOAD_BASE}/{tag}/{filename}");
    if asset.browser_download_url != expected_url {
        return Err(format!("URL inattendue pour l’asset {filename}."));
    }
    Ok(expected_url)
}

async fn read_limited(response: reqwest::Response, limit: usize) -> Result<Vec<u8>, String> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(format!(
            "Le fichier distant dépasse la limite de {limit} octets."
        ));
    }
    let response = response;
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| format!("Téléchargement interrompu : {error}"))?;
        if bytes.len().saturating_add(chunk.len()) > limit {
            return Err(format!(
                "Le fichier distant dépasse la limite de {limit} octets."
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag_name: &str, draft: bool, prerelease: bool) -> GitHubRelease {
        GitHubRelease {
            tag_name: tag_name.to_owned(),
            draft,
            prerelease,
            assets: Vec::new(),
        }
    }

    #[test]
    fn chooses_only_the_highest_published_dataset_sequence() {
        let releases = vec![
            release("rgaa-data-2", false, false),
            release("rgaa-data-9", true, false),
            release("rgaa-data-8", false, true),
            release("v1.0", false, false),
            release("rgaa-data-4", false, false),
        ];
        let (sequence, release) = latest_release(releases).expect("published dataset release");
        assert_eq!(sequence, 4);
        assert_eq!(release.tag_name, "rgaa-data-4");
    }

    #[test]
    fn release_downloads_are_limited_to_expected_github_assets() {
        let mut release = release("rgaa-data-4", false, false);
        release.assets.push(GitHubAsset {
            name: "manifest.json".to_owned(),
            browser_download_url:
                "https://github.com/All-Inclusives/RGAA-BIBLE/releases/download/rgaa-data-4/manifest.json"
                    .to_owned(),
        });
        assert!(download_url(&release, "rgaa-data-4", "manifest.json").is_ok());
        release.assets[0].browser_download_url = "https://example.com/manifest.json".to_owned();
        assert!(download_url(&release, "rgaa-data-4", "manifest.json").is_err());
    }
}
