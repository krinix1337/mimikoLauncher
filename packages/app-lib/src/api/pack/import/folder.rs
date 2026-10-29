//! Imports a single instance folder copied out of another launcher, such as a Prism Launcher
//! instance or a Modrinth App profile, which has no launcher metadata next to it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::bail;
use sqlx::{ConnectOptions, sqlite::SqliteConnectOptions};

use super::mmc;
use crate::api::pack::import;
use crate::ErrorKind;
use crate::State;
use crate::api::pack::install_from::{
    self, CreatePackDescription, PackDependency,
};
use crate::install::{InstallPhaseDetails, InstallProgressReporter};

/// Files and folders that only appear inside a Minecraft game directory.
const GAME_DIR_MARKERS: &[&str] = &[
    "options.txt",
    "mods",
    "saves",
    "resourcepacks",
    "shaderpacks",
    "config",
    "servers.dat",
];

/// Whether a folder is a Prism Launcher or MultiMC instance.
async fn is_mmc_instance(path: &Path) -> bool {
    mmc::is_valid_mmc(path.to_path_buf()).await
}

/// Whether a folder is a plain game directory, as Modrinth App and most launchers store them.
fn is_game_dir(path: &Path) -> bool {
    path.is_dir()
        && GAME_DIR_MARKERS
            .iter()
            .filter(|marker| path.join(marker).exists())
            .count()
            >= 2
}

/// Whether a folder can be imported on its own.
pub async fn is_instance_folder(path: &Path) -> bool {
    is_mmc_instance(path).await || is_game_dir(path)
}

/// Lists importable instance folders: the folder itself, or the instance folders inside it.
pub async fn list_instance_folders(path: &Path) -> Vec<String> {
    if is_instance_folder(path).await {
        return path
            .file_name()
            .map(|name| vec![name.to_string_lossy().into_owned()])
            .unwrap_or_default();
    }
    let mut folders = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(path).await else {
        return folders;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let child = entry.path();
        if child.is_dir() && is_instance_folder(&child).await {
            folders.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    folders.sort();
    folders
}

/// Resolves the folder chosen in the import list to its path on disk.
pub async fn resolve_instance_folder(
    base_path: &Path,
    instance_folder: &str,
) -> Option<PathBuf> {
    if base_path.file_name().is_some_and(|name| name == instance_folder)
        && is_instance_folder(base_path).await
    {
        return Some(base_path.to_path_buf());
    }
    let child = base_path.join(instance_folder);
    (is_instance_folder(&child).await).then_some(child)
}

pub async fn import_folder(
    path: PathBuf,
    instance_id: &str,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    if is_mmc_instance(&path).await {
        let mut icon_dirs = Vec::new();
        for ancestor in path.ancestors().skip(1).take(2) {
            icon_dirs.push(ancestor.join("icons"));
        }
        if let Some(data_dir) = dirs::data_dir() {
            for launcher in ["PrismLauncher", "ElyPrismLauncher", "MultiMC"] {
                icon_dirs.push(data_dir.join(launcher).join("icons"));
            }
        }
        return mmc::import_mmc_instance(
            path,
            &icon_dirs,
            instance_id,
            reporter,
            details,
        )
        .await;
    }

    let folder_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Imported instance".to_string());
    let known = find_in_modrinth_app(&folder_name).await;
    let name = known
        .as_ref()
        .map(|profile| profile.name.clone())
        .unwrap_or_else(|| folder_name.clone());
    let icon = match known.as_ref().and_then(|profile| profile.icon.clone()) {
        Some(icon) => import::recache_icon(icon).await?,
        None => find_folder_icon(&path).await?,
    };

    let game_version = match known
        .as_ref()
        .and_then(|profile| profile.game_version.clone())
    {
        Some(version) => version,
        None => detect_game_version(&path, &folder_name).await?,
    };
    let (loader, loader_version) = match known.as_ref() {
        Some(profile) if profile.loader.is_some() => (
            profile.loader.clone().unwrap_or_default(),
            profile.loader_version.clone(),
        ),
        _ => (detect_loader(&path).await.to_string(), None),
    };

    let mut dependencies = HashMap::new();
    dependencies.insert(PackDependency::Minecraft, game_version);
    let loader_version = loader_version.unwrap_or_else(|| "stable".to_string());
    match loader.as_str() {
        "fabric" => {
            dependencies.insert(PackDependency::FabricLoader, loader_version);
        }
        "quilt" => {
            dependencies.insert(PackDependency::QuiltLoader, loader_version);
        }
        "forge" => {
            dependencies.insert(PackDependency::Forge, loader_version);
        }
        "neoforge" => {
            dependencies.insert(PackDependency::NeoForge, loader_version);
        }
        _ => {}
    }

    let description = CreatePackDescription {
        icon,
        override_title: Some(name),
        project_id: None,
        version_id: None,
        instance_id: instance_id.to_string(),
        source_filename: None,
    };
    let state = State::get().await?;
    install_from::set_instance_information(
        instance_id.to_string(),
        &description,
        "Imported instance",
        None,
        &dependencies,
        false,
        &state.directories,
    )
    .await?;
    import::finish_import(instance_id, path, &state, reporter, details).await
}

struct KnownProfile {
    name: String,
    icon: Option<PathBuf>,
    game_version: Option<String>,
    loader: Option<String>,
    loader_version: Option<String>,
}

/// Looks the folder up in an installed Modrinth App, which keeps the name, versions and icon of
/// its instances in a database rather than inside the instance folder.
async fn find_in_modrinth_app(folder_name: &str) -> Option<KnownProfile> {
    let database = dirs::data_dir()?.join("ModrinthApp").join("app.db");
    if !database.is_file() {
        return None;
    }
    let mut connection = SqliteConnectOptions::new()
        .filename(&database)
        .read_only(true)
        .immutable(true)
        .connect()
        .await
        .ok()?;
    let row: Option<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT i.name, i.icon_path, c.game_version, c.loader, c.loader_version
		FROM instances i
		LEFT JOIN instance_content_sets c ON c.id = i.applied_content_set_id
		WHERE i.path = ?",
    )
    .bind(folder_name)
    .fetch_optional(&mut connection)
    .await
    .ok()
    .flatten();
    let row = match row {
        Some(row) => Some(row),
        None => sqlx::query_as(
            "SELECT name, icon_path, game_version, mod_loader, mod_loader_version
			FROM profiles WHERE path = ?",
        )
        .bind(folder_name)
        .fetch_optional(&mut connection)
        .await
        .ok()
        .flatten(),
    }?;
    let (name, icon, game_version, loader, loader_version) = row;
    Some(KnownProfile {
        name,
        icon: icon.map(PathBuf::from).filter(|icon| icon.is_file()),
        game_version,
        loader: loader.map(|loader| loader.to_lowercase()),
        loader_version,
    })
}

async fn find_folder_icon(path: &Path) -> crate::Result<Option<PathBuf>> {
    for name in ["icon.png", "icon.jpg", "pack.png"] {
        let icon = path.join(name);
        if icon.is_file() {
            return import::recache_icon(icon).await;
        }
    }
    Ok(None)
}

async fn detect_loader(path: &Path) -> &'static str {
    if path.join(".fabric").is_dir() {
        return "fabric";
    }
    if path.join(".quilt").is_dir() {
        return "quilt";
    }
    let config = path.join("config");
    if let Ok(mut entries) = tokio::fs::read_dir(&config).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.starts_with("neoforge-") {
                return "neoforge";
            }
            if name.starts_with("forge-") || name == "fml.toml" {
                return "forge";
            }
        }
    }
    for name in mod_file_names(path).await {
        let name = name.to_lowercase();
        if name.starts_with("fabric-api") || name.contains("-fabric") {
            return "fabric";
        }
        if name.contains("-neoforge") {
            return "neoforge";
        }
        if name.contains("-forge") {
            return "forge";
        }
    }
    "vanilla"
}

async fn mod_file_names(path: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(path.join("mods")).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names
}

/// Guesses the Minecraft version from the folder and mod file names, keeping only real versions.
async fn detect_game_version(
    path: &Path,
    folder_name: &str,
) -> crate::Result<String> {
    let manifest = crate::api::metadata::get_minecraft_versions().await?;
    let known: HashMap<&str, usize> = manifest
        .versions
        .iter()
        .enumerate()
        .map(|(index, version)| (version.id.as_str(), index))
        .collect();
    let mut votes: HashMap<String, usize> = HashMap::new();
    for candidate in version_tokens(folder_name) {
        *votes.entry(candidate).or_default() += 5;
    }
    for name in mod_file_names(path).await {
        for candidate in version_tokens(&name) {
            *votes.entry(candidate).or_default() += 1;
        }
    }
    let best = votes
        .into_iter()
        .filter(|(version, _)| known.contains_key(version.as_str()))
        .max_by(|(a, a_votes), (b, b_votes)| {
            a_votes
                .cmp(b_votes)
                .then_with(|| known[b.as_str()].cmp(&known[a.as_str()]))
        })
        .map(|(version, _)| version);
    match best {
        Some(version) => Ok(version),
        None => bail!(ErrorKind::InputError(format!(
            "could not detect the Minecraft version of '{folder_name}'. Add the version to the folder name, for example 'My pack 1.21.4'"
        ))),
    }
}

/// Finds strings shaped like Minecraft release versions, such as `1.21.4` or `26.1.2`.
fn version_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0;
    while start < bytes.len() {
        if !bytes[start].is_ascii_digit()
            || (start > 0
                && (bytes[start - 1].is_ascii_digit() || bytes[start - 1] == b'.'))
        {
            start += 1;
            continue;
        }
        let mut end = start;
        while end < bytes.len()
            && (bytes[end].is_ascii_digit() || bytes[end] == b'.')
        {
            end += 1;
        }
        let token = text[start..end].trim_end_matches('.');
        let parts: Vec<&str> = token.split('.').collect();
        if (2..=3).contains(&parts.len())
            && parts.iter().all(|part| !part.is_empty() && part.len() <= 2)
        {
            tokens.push(token.to_string());
        }
        start = end.max(start + 1);
    }
    tokens
}
