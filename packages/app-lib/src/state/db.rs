use crate::ErrorKind;
use crate::state::DirectoryInfo;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions,
};
use sqlx::{Pool, Sqlite};
use std::path::Path;
use std::time::Duration;

pub(crate) async fn connect(
    app_identifier: &str,
) -> crate::Result<Pool<Sqlite>> {
    let settings_dir = DirectoryInfo::initial_settings_dir_path(app_identifier)
        .ok_or(ErrorKind::FSError(
            "could not find valid config dir".to_string(),
        ))?;

    crate::util::io::create_dir_all(&settings_dir).await?;

    let db_path = settings_dir.join("app.db");

    connect_app_db(&db_path).await
}

async fn connect_app_db(db_path: &Path) -> crate::Result<Pool<Sqlite>> {
    if let Err(err) =
        super::db_backup::maybe_backup_existing_app_db(db_path).await
    {
        tracing::warn!(
            "Continuing without a pre-migration app database backup: {err}"
        );
    }
    open_migrated_app_db(db_path).await
}

/// How many rolling copies of the app database are kept.
const ROLLING_BACKUPS_KEPT: usize = 5;

/// Minimum time between two rolling copies, so restarts in quick succession
/// do not push out the older, possibly better, copies.
const ROLLING_BACKUP_INTERVAL: Duration = Duration::from_secs(10 * 60);

/// Copies the app database into a `backups` folder next to it, but only while
/// it holds user data, so an empty database never replaces a good copy.
async fn rolling_backup(pool: &Pool<Sqlite>, db_path: &Path) {
    let has_data = sqlx::query_scalar::<_, i64>(
        "SELECT (SELECT COUNT(*) FROM instances) + (SELECT COUNT(*) FROM minecraft_users)",
    )
    .fetch_one(pool)
    .await
    .is_ok_and(|count| count > 0);
    if !has_data {
        return;
    }

    let Some(backup_dir) = db_path.parent().map(|dir| dir.join("backups"))
    else {
        return;
    };
    if let Err(err) = std::fs::create_dir_all(&backup_dir) {
        tracing::warn!("Failed to create the database backup folder: {err}");
        return;
    }

    let newest_is_recent = list_rolling_backups(&backup_dir)
        .last()
        .and_then(|entry| entry.metadata().ok()?.modified().ok())
        .and_then(|modified| modified.elapsed().ok())
        .is_some_and(|age| age < ROLLING_BACKUP_INTERVAL);
    if newest_is_recent {
        return;
    }

    let backup_path = backup_dir.join(format!(
        "app-{}.db",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    ));
    if let Err(err) = sqlx::query("VACUUM INTO ?")
        .bind(backup_path.to_string_lossy().to_string())
        .execute(pool)
        .await
    {
        tracing::warn!("Failed to back up the app database: {err}");
        return;
    }

    let backups = list_rolling_backups(&backup_dir);
    let excess = backups.len().saturating_sub(ROLLING_BACKUPS_KEPT);
    for old in backups.into_iter().take(excess) {
        let _ = std::fs::remove_file(old.path());
    }
}

/// Lists the rolling backups in a folder, oldest first.
fn list_rolling_backups(backup_dir: &Path) -> Vec<std::fs::DirEntry> {
    let mut backups: Vec<_> = std::fs::read_dir(backup_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("app-"))
        .collect();
    backups.sort_by_key(|entry| entry.file_name());
    backups
}

async fn open_migrated_app_db(db_path: &Path) -> crate::Result<Pool<Sqlite>> {
    let pool = open_app_db_pool(db_path).await?;

    if let Err(err) = stale_data_cleanup(&pool).await {
        tracing::warn!(
            "Failed to clean up stale data from state database before migrations: {err}"
        );
    }

    sqlx::migrate!().run(&pool).await?;
    record_current_app_version(&pool).await?;
    rolling_backup(&pool, db_path).await;

    if let Err(err) = stale_data_cleanup(&pool).await {
        tracing::warn!(
            "Failed to clean up stale data from state database: {err}"
        );
    }

    Ok(pool)
}

async fn open_app_db_pool(db_path: &Path) -> crate::Result<Pool<Sqlite>> {
    let conn_options = SqliteConnectOptions::new()
        .filename(db_path)
        .busy_timeout(Duration::from_secs(30))
        .journal_mode(SqliteJournalMode::Wal)
        .optimize_on_close(true, None)
        .create_if_missing(true);

    Ok(SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .idle_timeout(Duration::from_secs(120))
        .connect_with(conn_options)
        .await?)
}

async fn record_current_app_version(pool: &Pool<Sqlite>) -> crate::Result<()> {
    let previous_version = sqlx::query_scalar!(
        "SELECT value FROM app_metadata WHERE key = 'app_version'"
    )
    .fetch_optional(pool)
    .await?;
    let already_used_sync_update = previous_version
        .as_deref()
        .and_then(|version| {
            let mut parts = version.split('.');
            Some((
                parts.next()?.parse::<u64>().ok()?,
                parts.next()?.parse::<u64>().ok()?,
            ))
        })
        .is_some_and(|version| version >= (0, 20));

    if env!("CARGO_PKG_VERSION").starts_with("0.20.")
        && already_used_sync_update
    {
        let mut settings = super::Settings::get(pool).await?;
        if settings.pending_update_toast_for_version.is_some() {
            settings.pending_update_toast_for_version = None;
            settings.update(pool).await?;
        }
    }

    sqlx::query!(
        "
		INSERT INTO app_metadata (key, value, updated_at)
		VALUES ('app_version', ?, unixepoch())
		ON CONFLICT(key) DO UPDATE SET
			value = excluded.value,
			updated_at = excluded.updated_at
		",
        env!("CARGO_PKG_VERSION"),
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Cleans up data from the database that is no longer referenced, but must be
/// kept around for a little while to allow users to recover from accidental
/// deletions.
async fn stale_data_cleanup(pool: &Pool<Sqlite>) -> crate::Result<()> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let has_skin_tables = sqlx::query!(
		"SELECT COUNT(*) AS \"count!: i64\" FROM sqlite_master WHERE type = 'table' AND name IN ('custom_minecraft_skins', 'minecraft_users')",
	)
	.fetch_one(&mut *tx)
	.await?
	.count == 2;

    if has_skin_tables {
        sqlx::query!(
			"DELETE FROM custom_minecraft_skins WHERE minecraft_user_uuid NOT IN (SELECT uuid FROM minecraft_users)"
		)
		.execute(&mut *tx)
		.await?;
    }

    tx.commit().await?;

    Ok(())
}
