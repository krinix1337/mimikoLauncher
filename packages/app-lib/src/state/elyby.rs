//! Ely.by accounts: Yggdrasil authentication against `authserver.ely.by` and
//! the authlib-injector agent that makes the game use Ely.by for skins and
//! server joins.

use std::path::PathBuf;

use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::ErrorKind;
use crate::state::{Credentials, DirectoryInfo, MinecraftProfile};
use crate::util::fetch::REQWEST_CLIENT;

const AUTH_SERVER: &str = "https://authserver.ely.by/auth";
const AUTHLIB_INJECTOR_LATEST: &str =
    "https://authlib-injector.yushi.moe/artifact/latest.json";
const AUTHLIB_INJECTOR_FILE: &str = "authlib-injector.jar";

/// Prefix of the refresh token of Ely.by accounts; the rest is the client token.
pub const ELYBY_REFRESH_PREFIX: &str = "elyby:";

/// How long an Ely.by access token is used before it is refreshed.
const TOKEN_REFRESH_INTERVAL: i64 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthResponse {
    access_token: String,
    client_token: String,
    selected_profile: Option<AuthProfile>,
}

#[derive(Deserialize)]
struct AuthProfile {
    id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthError {
    error_message: Option<String>,
}

async fn post_auth(
    endpoint: &str,
    body: serde_json::Value,
) -> crate::Result<AuthResponse> {
    let response = REQWEST_CLIENT
        .post(format!("{AUTH_SERVER}/{endpoint}"))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let message = response
            .json::<AuthError>()
            .await
            .ok()
            .and_then(|error| error.error_message)
            .unwrap_or_default();

        let message = if message.contains("two factor") {
            "Включена двухфакторная защита: введи пароль в виде пароль:код"
                .to_string()
        } else if message.contains("Invalid credentials") {
            "Неверный логин или пароль Ely.by".to_string()
        } else if message.is_empty() {
            "Не удалось войти через Ely.by".to_string()
        } else {
            format!("Ely.by: {message}")
        };

        return Err(ErrorKind::InputError(message).into());
    }

    Ok(response.json().await?)
}

/// Signs into an Ely.by account and stores it as the active account.
#[tracing::instrument(skip(password))]
pub async fn login_elyby(
    username: &str,
    password: &str,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<Credentials> {
    let client_token = Uuid::new_v4().simple().to_string();
    let response = post_auth(
        "authenticate",
        json!({
            "username": username.trim(),
            "password": password,
            "clientToken": client_token,
            "requestUser": true,
        }),
    )
    .await?;

    let profile = response.selected_profile.ok_or_else(|| {
        ErrorKind::InputError(
            "У аккаунта Ely.by нет игрового профиля".to_string(),
        )
    })?;
    let id = Uuid::parse_str(&profile.id).map_err(|_| {
        ErrorKind::InputError(
            "Ely.by вернул некорректный UUID профиля".to_string(),
        )
    })?;

    let credentials = Credentials {
        offline_profile: MinecraftProfile {
            id,
            name: profile.name,
            ..MinecraftProfile::default()
        },
        access_token: response.access_token,
        refresh_token: format!(
            "{ELYBY_REFRESH_PREFIX}{}",
            response.client_token
        ),
        expires: Utc::now() + Duration::days(TOKEN_REFRESH_INTERVAL),
        active: true,
    };

    credentials.upsert(exec).await?;

    Ok(credentials)
}

impl Credentials {
    /// Whether these credentials belong to an Ely.by account.
    pub fn is_elyby(&self) -> bool {
        self.refresh_token.starts_with(ELYBY_REFRESH_PREFIX)
    }

    /// Refreshes the Ely.by access token once it is due. Failures keep the
    /// current token, since Ely.by tokens stay valid for a long time and a
    /// network hiccup should not block launching.
    pub(super) async fn refresh_elyby(
        &mut self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    ) -> crate::Result<()> {
        if self.expires > Utc::now() + Duration::minutes(5) {
            return Ok(());
        }

        let client_token = self
            .refresh_token
            .trim_start_matches(ELYBY_REFRESH_PREFIX)
            .to_string();

        match post_auth(
            "refresh",
            json!({
                "accessToken": self.access_token,
                "clientToken": client_token,
            }),
        )
        .await
        {
            Ok(response) => {
                self.access_token = response.access_token;
                self.expires =
                    Utc::now() + Duration::days(TOKEN_REFRESH_INTERVAL);
                self.upsert(exec).await?;
            }
            Err(error) => {
                tracing::warn!("Failed to refresh Ely.by token: {error}");
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
struct AuthlibInjectorArtifact {
    download_url: String,
}

/// Returns the path of the authlib-injector agent, downloading it on first use.
pub async fn authlib_injector_path(
    directories: &DirectoryInfo,
) -> crate::Result<PathBuf> {
    let path = directories.caches_dir().join(AUTHLIB_INJECTOR_FILE);
    if tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Ok(path);
    }

    let artifact: AuthlibInjectorArtifact = REQWEST_CLIENT
        .get(AUTHLIB_INJECTOR_LATEST)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let jar = REQWEST_CLIENT
        .get(&artifact.download_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, &jar).await?;

    Ok(path)
}
