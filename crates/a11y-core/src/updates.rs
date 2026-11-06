use std::{
    fmt, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{A11yCore, CoreError};

const SCHEMA_VERSION: &str = "1.0.0";
static STAGING_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateManifest {
    pub schema_version: String,
    pub sequence: u64,
    pub rgaa_version: String,
    pub source_commit: String,
    pub package_file: String,
    pub package_sha256: String,
    pub package_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl UpdateManifest {
    /// Builds release metadata for a validated package, using the supplied
    /// sequence as its monotonic release number.
    pub fn for_package(sequence: u64, package_bytes: &[u8]) -> Result<Self, UpdateError> {
        let core = A11yCore::from_json_str(
            std::str::from_utf8(package_bytes)
                .map_err(|error| UpdateError::InvalidPackage(error.to_string()))?,
        )
        .map_err(UpdateError::Dataset)?;

        if sequence == 0 {
            return Err(UpdateError::InvalidManifest(
                "sequence must be greater than zero".to_owned(),
            ));
        }

        Ok(Self {
            schema_version: SCHEMA_VERSION.to_owned(),
            sequence,
            rgaa_version: core.dataset().source.rgaa_version.clone(),
            source_commit: core.dataset().source.commit.clone(),
            package_file: format!("rgaa-{}.json", core.dataset().source.rgaa_version),
            package_sha256: sha256_hex(package_bytes),
            package_bytes: package_bytes.len() as u64,
            signature: None,
        })
    }

    /// Returns the stable byte representation authenticated by the Ed25519
    /// signature. The signature itself is intentionally excluded.
    pub fn signing_bytes(&self) -> Vec<u8> {
        format!(
            "AccessiGuide RGAA update manifest v1\nschemaVersion:{}\nsequence:{}\nrgaaVersion:{}\nsourceCommit:{}\npackageFile:{}\npackageSha256:{}\npackageBytes:{}\n",
            self.schema_version,
            self.sequence,
            self.rgaa_version,
            self.source_commit,
            self.package_file,
            self.package_sha256,
            self.package_bytes
        )
        .into_bytes()
    }

    /// Verifies this manifest using the application's trusted 32-byte
    /// Ed25519 public key, supplied as hexadecimal in release configuration.
    pub fn verify_signature(&self, trusted_public_key: &[u8; 32]) -> Result<(), UpdateError> {
        let public_key = VerifyingKey::from_bytes(trusted_public_key)
            .map_err(|error| UpdateError::InvalidSignature(error.to_string()))?;
        let signature_bytes = self
            .signature
            .as_deref()
            .ok_or_else(|| UpdateError::InvalidSignature("signature is missing".to_owned()))
            .and_then(decode_hex_signature)?;
        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|error| UpdateError::InvalidSignature(error.to_string()))?;
        public_key
            .verify(&self.signing_bytes(), &signature)
            .map_err(|error| UpdateError::InvalidSignature(error.to_string()))
    }

    pub fn to_json(&self) -> Result<String, UpdateError> {
        serde_json::to_string_pretty(self).map_err(|error| {
            UpdateError::InvalidManifest(format!("cannot encode manifest: {error}"))
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledUpdate {
    pub sequence: u64,
    pub rgaa_version: String,
    pub source_commit: String,
    pub directory: PathBuf,
}

#[derive(Debug)]
pub enum UpdateError {
    Io(std::io::Error),
    InvalidManifest(String),
    ChecksumMismatch { expected: String, actual: String },
    InvalidPackage(String),
    Dataset(CoreError),
    InvalidSignature(String),
    OlderSequence { current: u64, incoming: u64 },
    SequenceConflict(u64),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "update storage I/O failed: {error}"),
            Self::InvalidManifest(message) => {
                write!(formatter, "invalid update manifest: {message}")
            }
            Self::ChecksumMismatch { expected, actual } => write!(
                formatter,
                "package checksum mismatch: expected {expected}, received {actual}"
            ),
            Self::InvalidPackage(message) => write!(formatter, "invalid update package: {message}"),
            Self::Dataset(error) => write!(formatter, "update dataset rejected: {error}"),
            Self::InvalidSignature(message) => {
                write!(formatter, "update manifest signature rejected: {message}")
            }
            Self::OlderSequence { current, incoming } => write!(
                formatter,
                "update sequence {incoming} is older than installed sequence {current}"
            ),
            Self::SequenceConflict(sequence) => write!(
                formatter,
                "an update with sequence {sequence} is already installed with different content"
            ),
        }
    }
}

impl std::error::Error for UpdateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Dataset(error) => Some(error),
            Self::InvalidManifest(_)
            | Self::ChecksumMismatch { .. }
            | Self::InvalidPackage(_)
            | Self::InvalidSignature(_)
            | Self::OlderSequence { .. }
            | Self::SequenceConflict(_) => None,
        }
    }
}

impl From<std::io::Error> for UpdateError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Validates a package fully, then installs it as an immutable versioned
/// directory. The final same-volume directory rename is the activation point;
/// incomplete staging directories are never considered installed. The checksum
/// proves package integrity only; the manifest must come from a trusted source.
pub fn install_update(
    store_dir: impl AsRef<Path>,
    manifest_json: &[u8],
    package_bytes: &[u8],
) -> Result<InstalledUpdate, UpdateError> {
    let store_dir = store_dir.as_ref();
    let manifest = parse_manifest(manifest_json)?;
    validate_package(&manifest, package_bytes)?;
    let current = latest_manifest(store_dir)?;

    if let Some((current_dir, current_manifest)) = current {
        if manifest.sequence < current_manifest.sequence {
            return Err(UpdateError::OlderSequence {
                current: current_manifest.sequence,
                incoming: manifest.sequence,
            });
        }
        if manifest.sequence == current_manifest.sequence {
            if manifest == current_manifest && current_dir.exists() {
                let current_package = fs::read(current_dir.join(&current_manifest.package_file))?;
                validate_package(&current_manifest, &current_package)?;
                return Ok(installed_update(current_dir, &manifest));
            }
            return Err(UpdateError::SequenceConflict(manifest.sequence));
        }
    }

    fs::create_dir_all(store_dir)?;
    let final_dir = store_dir.join(update_directory_name(manifest.sequence));
    if final_dir.exists() {
        return Err(UpdateError::SequenceConflict(manifest.sequence));
    }

    let staging_dir = next_staging_directory(store_dir, manifest.sequence);
    fs::create_dir(&staging_dir)?;
    let result = write_staged_update(&staging_dir, &manifest, manifest_json, package_bytes)
        .and_then(|()| {
            fs::rename(&staging_dir, &final_dir)?;
            Ok(())
        });
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging_dir);
    }
    result?;

    Ok(installed_update(final_dir, &manifest))
}

/// Verifies a signed manifest against the trusted public key before allowing
/// the package to be installed.
pub fn install_signed_update(
    store_dir: impl AsRef<Path>,
    manifest_json: &[u8],
    package_bytes: &[u8],
    trusted_public_key: &[u8; 32],
) -> Result<InstalledUpdate, UpdateError> {
    let manifest = parse_manifest(manifest_json)?;
    manifest.verify_signature(trusted_public_key)?;
    install_update(store_dir, manifest_json, package_bytes)
}

/// Loads and verifies the highest-sequence installed update, if one exists.
pub fn load_from_update_store(
    store_dir: impl AsRef<Path>,
) -> Result<Option<A11yCore>, UpdateError> {
    let Some((directory, manifest)) = latest_manifest(store_dir.as_ref())? else {
        return Ok(None);
    };
    let package_bytes = fs::read(directory.join(&manifest.package_file))?;
    validate_package(&manifest, &package_bytes).map(Some)
}

/// Returns metadata for the highest-sequence installed update, if one exists.
pub fn latest_installed_update(
    store_dir: impl AsRef<Path>,
) -> Result<Option<InstalledUpdate>, UpdateError> {
    Ok(latest_manifest(store_dir.as_ref())?
        .map(|(directory, manifest)| installed_update(directory, &manifest)))
}

/// Uses the newest validated local update, or the embedded corpus when the
/// update store has no installed releases. Store errors are surfaced.
pub fn load_or_bundled(store_dir: impl AsRef<Path>) -> Result<A11yCore, UpdateError> {
    match load_from_update_store(store_dir)? {
        Some(core) => Ok(core),
        None => A11yCore::from_bundled().map_err(UpdateError::Dataset),
    }
}

/// Loads and verifies the latest installed signed update, or the bundled
/// corpus if no update has been installed.
pub fn load_signed_or_bundled(
    store_dir: impl AsRef<Path>,
    trusted_public_key: &[u8; 32],
) -> Result<A11yCore, UpdateError> {
    let Some((directory, manifest)) = latest_manifest(store_dir.as_ref())? else {
        return A11yCore::from_bundled().map_err(UpdateError::Dataset);
    };
    manifest.verify_signature(trusted_public_key)?;
    let package_bytes = fs::read(directory.join(&manifest.package_file))?;
    validate_package(&manifest, &package_bytes)
}

fn parse_manifest(json: &[u8]) -> Result<UpdateManifest, UpdateError> {
    let manifest: UpdateManifest = serde_json::from_slice(json)
        .map_err(|error| UpdateError::InvalidManifest(error.to_string()))?;
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(UpdateError::InvalidManifest(format!(
            "unsupported schema version {}",
            manifest.schema_version
        )));
    }
    if manifest.sequence == 0 {
        return Err(UpdateError::InvalidManifest(
            "sequence must be greater than zero".to_owned(),
        ));
    }
    if manifest.package_bytes == 0 {
        return Err(UpdateError::InvalidManifest(
            "packageBytes must be greater than zero".to_owned(),
        ));
    }
    if manifest.source_commit.len() != 40
        || !manifest
            .source_commit
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(UpdateError::InvalidManifest(
            "sourceCommit must be a 40-character Git hash".to_owned(),
        ));
    }
    if manifest.package_file != format!("rgaa-{}.json", manifest.rgaa_version)
        || Path::new(&manifest.package_file)
            .file_name()
            .and_then(|name| name.to_str())
            != Some(manifest.package_file.as_str())
    {
        return Err(UpdateError::InvalidManifest(
            "packageFile must be a local RGAA package filename".to_owned(),
        ));
    }
    if manifest.package_sha256.len() != 64
        || !manifest
            .package_sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(UpdateError::InvalidManifest(
            "packageSha256 must be a 64-character hexadecimal digest".to_owned(),
        ));
    }
    Ok(manifest)
}

fn validate_package(
    manifest: &UpdateManifest,
    package_bytes: &[u8],
) -> Result<A11yCore, UpdateError> {
    if manifest.package_bytes != package_bytes.len() as u64 {
        return Err(UpdateError::InvalidPackage(format!(
            "package size mismatch: expected {}, received {}",
            manifest.package_bytes,
            package_bytes.len()
        )));
    }
    let actual_digest = sha256_hex(package_bytes);
    if !actual_digest.eq_ignore_ascii_case(&manifest.package_sha256) {
        return Err(UpdateError::ChecksumMismatch {
            expected: manifest.package_sha256.clone(),
            actual: actual_digest,
        });
    }
    let package = std::str::from_utf8(package_bytes)
        .map_err(|error| UpdateError::InvalidPackage(error.to_string()))?;
    let core = A11yCore::from_json_str(package).map_err(UpdateError::Dataset)?;
    if core.dataset().source.rgaa_version != manifest.rgaa_version
        || core.dataset().source.commit != manifest.source_commit
    {
        return Err(UpdateError::InvalidManifest(
            "package provenance does not match manifest".to_owned(),
        ));
    }
    Ok(core)
}

fn latest_manifest(store_dir: &Path) -> Result<Option<(PathBuf, UpdateManifest)>, UpdateError> {
    if !store_dir.exists() {
        return Ok(None);
    }
    let mut latest: Option<(PathBuf, UpdateManifest)> = None;
    for entry in fs::read_dir(store_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(sequence) = name
            .strip_prefix("update-")
            .and_then(|value| value.parse::<u64>().ok())
        else {
            continue;
        };
        let directory = entry.path();
        let manifest = parse_manifest(&fs::read(directory.join("manifest.json"))?)?;
        if manifest.sequence != sequence {
            return Err(UpdateError::InvalidManifest(format!(
                "directory sequence {sequence} does not match manifest sequence {}",
                manifest.sequence
            )));
        }
        if latest
            .as_ref()
            .is_none_or(|(_, latest_manifest)| sequence > latest_manifest.sequence)
        {
            latest = Some((directory, manifest));
        }
    }
    Ok(latest)
}

fn write_staged_update(
    staging_dir: &Path,
    manifest: &UpdateManifest,
    manifest_json: &[u8],
    package_bytes: &[u8],
) -> Result<(), UpdateError> {
    let manifest_path = staging_dir.join("manifest.json");
    let package_path = staging_dir.join(&manifest.package_file);
    write_new_synced(&manifest_path, manifest_json)?;
    write_new_synced(&package_path, package_bytes)?;
    Ok(())
}

fn write_new_synced(path: &Path, contents: &[u8]) -> Result<(), UpdateError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

fn next_staging_directory(store_dir: &Path, sequence: u64) -> PathBuf {
    let id = STAGING_ID.fetch_add(1, Ordering::Relaxed);
    store_dir.join(format!(".staging-{sequence}-{}-{id}", std::process::id()))
}

fn update_directory_name(sequence: u64) -> String {
    format!("update-{sequence}")
}

fn installed_update(directory: PathBuf, manifest: &UpdateManifest) -> InstalledUpdate {
    InstalledUpdate {
        sequence: manifest.sequence,
        rgaa_version: manifest.rgaa_version.clone(),
        source_commit: manifest.source_commit.clone(),
        directory,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex_signature(encoded: &str) -> Result<Vec<u8>, UpdateError> {
    if encoded.len() != 128 || !encoded.is_ascii() {
        return Err(UpdateError::InvalidSignature(
            "signature must contain 128 hexadecimal characters".to_owned(),
        ));
    }
    let mut decoded = Vec::with_capacity(64);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = hex_digit(pair[0])?;
        let low = hex_digit(pair[1])?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

fn hex_digit(value: u8) -> Result<u8, UpdateError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(UpdateError::InvalidSignature(
            "signature contains a non-hexadecimal character".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    static TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn test_store() -> PathBuf {
        std::env::temp_dir().join(format!(
            "a11y-core-updates-{}-{}",
            std::process::id(),
            TEST_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn package_bytes() -> Vec<u8> {
        include_str!("../../../data/rgaa-4.1.2.json")
            .as_bytes()
            .to_vec()
    }

    fn manifest_bytes(sequence: u64, package: &[u8]) -> Vec<u8> {
        UpdateManifest::for_package(sequence, package)
            .unwrap()
            .to_json()
            .unwrap()
            .into_bytes()
    }

    fn signed_manifest_bytes(sequence: u64, package: &[u8]) -> (Vec<u8>, [u8; 32]) {
        let signing_key = SigningKey::from_bytes(&[42; 32]);
        let mut manifest = UpdateManifest::for_package(sequence, package).unwrap();
        let signature = signing_key.sign(&manifest.signing_bytes());
        manifest.signature = Some(
            signature
                .to_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        );
        (
            manifest.to_json().unwrap().into_bytes(),
            signing_key.verifying_key().to_bytes(),
        )
    }

    #[test]
    fn validates_integrity_and_installs_update_atomically() {
        let store = test_store();
        let package = package_bytes();
        let manifest = manifest_bytes(1, &package);
        let installed = install_update(&store, &manifest, &package).unwrap();
        assert_eq!(installed.sequence, 1);
        assert_eq!(
            latest_installed_update(&store)
                .unwrap()
                .unwrap()
                .sequence,
            1
        );
        assert_eq!(
            load_from_update_store(&store)
                .unwrap()
                .unwrap()
                .dataset()
                .counts
                .criteria,
            106
        );
        assert!(store.join(update_directory_name(1)).is_dir());
        assert!(fs::read_dir(&store).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".staging-")
        }));
        fs::remove_dir_all(store).unwrap();
    }

    #[test]
    fn rejects_corruption_and_preserves_the_active_update() {
        let store = test_store();
        let package = package_bytes();
        let first_manifest = manifest_bytes(1, &package);
        install_update(&store, &first_manifest, &package).unwrap();

        let mut corrupt_package = package.clone();
        corrupt_package[0] ^= 0x01;
        let error =
            install_update(&store, &manifest_bytes(2, &package), &corrupt_package).unwrap_err();
        assert!(matches!(
            error,
            UpdateError::InvalidPackage(_) | UpdateError::ChecksumMismatch { .. }
        ));
        assert_eq!(
            load_from_update_store(&store)
                .unwrap()
                .unwrap()
                .dataset()
                .source
                .commit,
            "ca4019f95073b6cbd2482a16e9f12b52d8de678d"
        );
        assert!(!store.join(update_directory_name(2)).exists());
        fs::remove_dir_all(store).unwrap();
    }

    #[test]
    fn rejects_unsupported_manifest_and_provenance_mismatch() {
        let package = package_bytes();
        let manifest: UpdateManifest =
            serde_json::from_slice(&manifest_bytes(1, &package)).unwrap();
        let mut unsupported = manifest.clone();
        unsupported.schema_version = "2.0.0".to_owned();
        assert!(matches!(
            parse_manifest(&serde_json::to_vec(&unsupported).unwrap()),
            Err(UpdateError::InvalidManifest(_))
        ));

        let mut mismatched = manifest;
        mismatched.source_commit = "0".repeat(40);
        assert!(matches!(
            validate_package(&mismatched, &package),
            Err(UpdateError::InvalidManifest(_))
        ));
    }

    #[test]
    fn requires_a_valid_trusted_signature_for_remote_updates() {
        let store = test_store();
        let package = package_bytes();
        let (manifest, trusted_key) = signed_manifest_bytes(1, &package);
        install_signed_update(&store, &manifest, &package, &trusted_key).unwrap();
        assert_eq!(
            load_signed_or_bundled(&store, &trusted_key)
                .unwrap()
                .dataset()
                .source
                .rgaa_version,
            "4.1.2"
        );

        let wrong_key = SigningKey::from_bytes(&[7; 32]).verifying_key().to_bytes();
        let rejected_store = test_store();
        assert!(matches!(
            install_signed_update(&rejected_store, &manifest, &package, &wrong_key),
            Err(UpdateError::InvalidSignature(_))
        ));
        assert!(!rejected_store.exists());
        fs::remove_dir_all(store).unwrap();
    }

    #[test]
    fn signed_startup_rejects_an_unsigned_installed_update() {
        let store = test_store();
        let package = package_bytes();
        let manifest = manifest_bytes(1, &package);
        install_update(&store, &manifest, &package).unwrap();
        let trusted_key = SigningKey::from_bytes(&[42; 32]).verifying_key().to_bytes();
        assert!(matches!(
            load_signed_or_bundled(&store, &trusted_key),
            Err(UpdateError::InvalidSignature(_))
        ));
        fs::remove_dir_all(store).unwrap();
    }

    #[test]
    fn installation_is_idempotent_and_rejects_old_or_conflicting_sequences() {
        let store = test_store();
        let package = package_bytes();
        let first_manifest = manifest_bytes(1, &package);
        let first = install_update(&store, &first_manifest, &package).unwrap();
        let repeated = install_update(&store, &first_manifest, &package).unwrap();
        assert_eq!(first, repeated);
        assert!(matches!(
            {
                let mut changed_dataset: serde_json::Value =
                    serde_json::from_slice(&package).unwrap();
                changed_dataset["topics"][0]["criteria"][0]["official"]["title"] =
                    serde_json::Value::String("Modified title".to_owned());
                let changed_package = serde_json::to_vec(&changed_dataset).unwrap();
                install_update(
                    &store,
                    &manifest_bytes(1, &changed_package),
                    &changed_package,
                )
            },
            Err(UpdateError::SequenceConflict(1))
        ));
        assert!(matches!(
            {
                install_update(&store, &manifest_bytes(2, &package), &package).unwrap();
                install_update(&store, &first_manifest, &package)
            },
            Err(UpdateError::OlderSequence { .. })
        ));
        fs::remove_dir_all(store).unwrap();
    }

    #[test]
    fn ignores_interrupted_staging_directories_and_uses_bundled_data_if_empty() {
        let store = test_store();
        fs::create_dir_all(store.join(".staging-1-interrupted")).unwrap();
        assert!(load_from_update_store(&store).unwrap().is_none());
        assert!(latest_installed_update(&store).unwrap().is_none());
        assert_eq!(
            load_or_bundled(&store).unwrap().dataset().counts.criteria,
            106
        );
        fs::remove_dir_all(store).unwrap();
    }
}
