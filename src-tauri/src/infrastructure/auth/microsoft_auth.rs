use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::application::ports::AuthenticationProviderPort;
use crate::domain::entities::{AccountType, MinecraftAccount, StoredMicrosoftAccount};
use crate::domain::errors::LauncherError;
use crate::domain::repositories::{MicrosoftAccountRepository, SettingsRepository};

const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const OAUTH_SCOPE: &str = "XboxLive.signin offline_access";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeInfo {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

struct MsTokenSet {
    access_token: String,
    refresh_token: String,
}

/// Implements Microsoft's device-code OAuth flow followed by the Xbox Live -> XSTS
/// -> Minecraft Services token exchange chain required to launch the game with a
/// real (premium) Minecraft account. See
/// https://minecraft.wiki/w/Microsoft_authentication for the reference flow.
pub struct MicrosoftAuthService {
    http: Client,
    repo: Arc<dyn MicrosoftAccountRepository>,
}

impl MicrosoftAuthService {
    pub fn new(repo: Arc<dyn MicrosoftAccountRepository>) -> Self {
        Self {
            http: Client::builder()
                .user_agent("NovaLauncher/1.0")
                .build()
                .unwrap_or_default(),
            repo,
        }
    }

    pub async fn get_stored_account(&self) -> Result<Option<StoredMicrosoftAccount>, LauncherError> {
        self.repo.get().await
    }

    pub async fn sign_out(&self) -> Result<(), LauncherError> {
        self.repo.clear().await
    }

    pub async fn begin_login(&self, client_id: &str) -> Result<DeviceCodeInfo, LauncherError> {
        let resp = self.http
            .post(DEVICE_CODE_URL)
            .form(&[("client_id", client_id), ("scope", OAUTH_SCOPE)])
            .send()
            .await
            .map_err(|e| LauncherError::network(format!("Failed to reach Microsoft sign-in: {}", e)))?;

        let status = resp.status();
        let body: Value = resp.json().await.map_err(|e| {
            LauncherError::network(format!("Invalid response from Microsoft sign-in: {}", e))
        })?;

        if !status.is_success() {
            return Err(LauncherError::validation(format!(
                "Microsoft sign-in could not start: {}",
                body.get("error_description").and_then(|v| v.as_str()).unwrap_or("unknown error")
            )));
        }

        Ok(DeviceCodeInfo {
            device_code: body.get("device_code").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            user_code: body.get("user_code").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            verification_uri: body.get("verification_uri").and_then(|v| v.as_str()).unwrap_or("https://microsoft.com/link").to_string(),
            expires_in: body.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(900),
            interval: body.get("interval").and_then(|v| v.as_u64()).unwrap_or(5),
        })
    }

    /// Polls the device code token endpoint until the user finishes signing in the
    /// browser, then runs the Xbox/Minecraft exchange chain and persists the account.
    pub async fn complete_login(
        &self,
        client_id: &str,
        device_code: &str,
        interval_secs: u64,
        expires_in_secs: u64,
    ) -> Result<MinecraftAccount, LauncherError> {
        let deadline = Instant::now() + Duration::from_secs(expires_in_secs);
        let mut interval = interval_secs.max(1);

        loop {
            if Instant::now() >= deadline {
                return Err(LauncherError::validation(
                    "Microsoft sign-in timed out - the code expired before you finished signing in. Please try again.",
                ));
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;

            let resp = self.http
                .post(TOKEN_URL)
                .form(&[
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("client_id", client_id),
                    ("device_code", device_code),
                ])
                .send()
                .await
                .map_err(|e| LauncherError::network(format!("Failed to reach Microsoft sign-in: {}", e)))?;

            let status = resp.status();
            let body: Value = resp.json().await.map_err(|e| {
                LauncherError::network(format!("Invalid response from Microsoft sign-in: {}", e))
            })?;

            if status.is_success() {
                let token_set = MsTokenSet {
                    access_token: body.get("access_token").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    refresh_token: body.get("refresh_token").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                };
                return self.finish_with_ms_token(token_set).await;
            }

            match body.get("error").and_then(|v| v.as_str()).unwrap_or_default() {
                "authorization_pending" => continue,
                "slow_down" => {
                    interval += 5;
                    continue;
                }
                "expired_token" => {
                    return Err(LauncherError::validation(
                        "The sign-in code expired before you finished. Please try again.",
                    ));
                }
                "authorization_declined" => {
                    return Err(LauncherError::validation("Sign-in was cancelled."));
                }
                other => {
                    return Err(LauncherError::validation(format!(
                        "Microsoft sign-in failed: {}",
                        body.get("error_description").and_then(|v| v.as_str()).unwrap_or(other)
                    )));
                }
            }
        }
    }

    /// Silently refreshes an expired Minecraft session using the stored MS refresh
    /// token, without requiring the user to sign in again through the browser.
    pub async fn refresh(&self, client_id: &str, stored: &StoredMicrosoftAccount) -> Result<MinecraftAccount, LauncherError> {
        let resp = self.http
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", client_id),
                ("refresh_token", stored.ms_refresh_token.as_str()),
                ("scope", OAUTH_SCOPE),
            ])
            .send()
            .await
            .map_err(|e| LauncherError::network(format!("Failed to reach Microsoft sign-in: {}", e)))?;

        let status = resp.status();
        let body: Value = resp.json().await.map_err(|e| {
            LauncherError::network(format!("Invalid response from Microsoft sign-in: {}", e))
        })?;

        if !status.is_success() {
            return Err(LauncherError::validation(
                "Your Microsoft session expired and could not be renewed automatically. Please sign in again from the account menu.",
            ));
        }

        let token_set = MsTokenSet {
            access_token: body.get("access_token").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            refresh_token: body.get("refresh_token").and_then(|v| v.as_str()).unwrap_or(&stored.ms_refresh_token).to_string(),
        };
        self.finish_with_ms_token(token_set).await
    }

    async fn finish_with_ms_token(&self, token_set: MsTokenSet) -> Result<MinecraftAccount, LauncherError> {
        let (xbl_token, user_hash) = self.xbox_live_authenticate(&token_set.access_token).await?;
        let xsts_token = self.xsts_authorize(&xbl_token).await?;
        let (mc_access_token, mc_expires_in) = self.minecraft_login(&xsts_token, &user_hash).await?;
        let (username, uuid) = self.fetch_profile(&mc_access_token).await?;

        let now = Utc::now();
        let stored = StoredMicrosoftAccount {
            username: username.clone(),
            uuid: uuid.clone(),
            minecraft_access_token: mc_access_token.clone(),
            minecraft_token_expires_at: now + ChronoDuration::seconds(mc_expires_in as i64),
            ms_refresh_token: token_set.refresh_token,
            created_at: now,
            updated_at: now,
        };
        self.repo.save(&stored).await?;

        Ok(MinecraftAccount {
            id: format!("microsoft-{}", uuid),
            username,
            uuid,
            access_token: mc_access_token,
            account_type: AccountType::Microsoft,
        })
    }

    async fn xbox_live_authenticate(&self, ms_access_token: &str) -> Result<(String, String), LauncherError> {
        let payload = serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", ms_access_token),
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
        });

        let resp = self.http
            .post(XBL_AUTH_URL)
            .json(&payload)
            .send()
            .await
            .map_err(|e| LauncherError::network(format!("Failed to reach Xbox Live: {}", e)))?;

        if !resp.status().is_success() {
            return Err(LauncherError::validation("Xbox Live rejected this Microsoft account."));
        }

        let body: Value = resp.json().await.map_err(|e| {
            LauncherError::network(format!("Invalid response from Xbox Live: {}", e))
        })?;

        Self::extract_token_and_hash(&body, "Xbox Live")
    }

    async fn xsts_authorize(&self, xbl_token: &str) -> Result<String, LauncherError> {
        let payload = serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl_token],
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
        });

        let resp = self.http
            .post(XSTS_AUTH_URL)
            .json(&payload)
            .send()
            .await
            .map_err(|e| LauncherError::network(format!("Failed to reach Xbox Live: {}", e)))?;

        let status = resp.status();
        let body: Value = resp.json().await.map_err(|e| {
            LauncherError::network(format!("Invalid response from Xbox Live: {}", e))
        })?;

        if !status.is_success() {
            let message = match body.get("XErr").and_then(|v| v.as_u64()) {
                Some(2148916233) => "This Microsoft account has no Xbox Live profile. Create one at xbox.com, then try again.".to_string(),
                Some(2148916235) => "Xbox Live is not available in this account's region.".to_string(),
                Some(2148916236) | Some(2148916237) => "This Microsoft account needs adult verification on xbox.com before it can sign in.".to_string(),
                Some(2148916238) => "This is a child account. An adult needs to add it to a Microsoft family group before it can sign in.".to_string(),
                _ => "Xbox Live rejected this Microsoft account.".to_string(),
            };
            return Err(LauncherError::validation(message));
        }

        let (token, _hash) = Self::extract_token_and_hash(&body, "Xbox Live")?;
        Ok(token)
    }

    fn extract_token_and_hash(body: &Value, source: &str) -> Result<(String, String), LauncherError> {
        let token = body.get("Token").and_then(|v| v.as_str()).ok_or_else(|| {
            LauncherError::minecraft(format!("{} response is missing a token", source), None)
        })?;
        let hash = body
            .pointer("/DisplayClaims/xui/0/uhs")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LauncherError::minecraft(format!("{} response is missing a user hash", source), None))?;
        Ok((token.to_string(), hash.to_string()))
    }

    async fn minecraft_login(&self, xsts_token: &str, user_hash: &str) -> Result<(String, u64), LauncherError> {
        let payload = serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token),
        });

        let resp = self.http
            .post(MC_LOGIN_URL)
            .json(&payload)
            .send()
            .await
            .map_err(|e| LauncherError::network(format!("Failed to reach Minecraft services: {}", e)))?;

        if !resp.status().is_success() {
            return Err(LauncherError::validation("Minecraft services rejected this login."));
        }

        let body: Value = resp.json().await.map_err(|e| {
            LauncherError::network(format!("Invalid response from Minecraft services: {}", e))
        })?;

        let access_token = body.get("access_token").and_then(|v| v.as_str()).ok_or_else(|| {
            LauncherError::minecraft("Minecraft services response is missing an access token", None)
        })?;
        let expires_in = body.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(86400);

        Ok((access_token.to_string(), expires_in))
    }

    async fn fetch_profile(&self, mc_access_token: &str) -> Result<(String, String), LauncherError> {
        let resp = self.http
            .get(MC_PROFILE_URL)
            .bearer_auth(mc_access_token)
            .send()
            .await
            .map_err(|e| LauncherError::network(format!("Failed to reach Minecraft services: {}", e)))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(LauncherError::validation(
                "This Microsoft account doesn't own Minecraft: Java Edition.",
            ));
        }
        if !resp.status().is_success() {
            return Err(LauncherError::validation("Could not fetch the Minecraft profile for this account."));
        }

        let body: Value = resp.json().await.map_err(|e| {
            LauncherError::network(format!("Invalid Minecraft profile response: {}", e))
        })?;

        let name = body.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            LauncherError::minecraft("Minecraft profile response is missing a username", None)
        })?;
        let raw_id = body.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
            LauncherError::minecraft("Minecraft profile response is missing a UUID", None)
        })?;

        Ok((name.to_string(), Self::format_uuid(raw_id)))
    }

    /// Mojang APIs return UUIDs without dashes; the launch arguments need the
    /// standard 8-4-4-4-12 hyphenated form.
    fn format_uuid(raw: &str) -> String {
        if raw.len() != 32 {
            return raw.to_string();
        }
        format!(
            "{}-{}-{}-{}-{}",
            &raw[0..8], &raw[8..12], &raw[12..16], &raw[16..20], &raw[20..32]
        )
    }
}

/// Bridges the interactive Microsoft auth service to the generic
/// `AuthenticationProviderPort` the launcher uses at launch time: returns the
/// cached account if its Minecraft token is still valid, silently refreshes it
/// otherwise, and surfaces a clear error if the user needs to sign in again.
pub struct MicrosoftAuthenticationProvider {
    service: Arc<MicrosoftAuthService>,
    settings_repo: Arc<dyn SettingsRepository>,
}

impl MicrosoftAuthenticationProvider {
    pub fn new(service: Arc<MicrosoftAuthService>, settings_repo: Arc<dyn SettingsRepository>) -> Self {
        Self { service, settings_repo }
    }
}

#[async_trait]
impl AuthenticationProviderPort for MicrosoftAuthenticationProvider {
    async fn get_active_account(&self) -> Result<MinecraftAccount, LauncherError> {
        let stored = self.service.get_stored_account().await?.ok_or_else(|| {
            LauncherError::validation("No Microsoft account is signed in. Please sign in from the account menu.")
        })?;

        if stored.minecraft_token_expires_at > Utc::now() + ChronoDuration::seconds(60) {
            return Ok(MinecraftAccount {
                id: format!("microsoft-{}", stored.uuid),
                username: stored.username,
                uuid: stored.uuid,
                access_token: stored.minecraft_access_token,
                account_type: AccountType::Microsoft,
            });
        }

        let settings = self.settings_repo.get().await?;
        let client_id = settings.microsoft_client_id.ok_or_else(|| {
            LauncherError::validation("Microsoft sign-in is not configured (missing Client ID in Settings).")
        })?;

        self.service.refresh(&client_id, &stored).await
    }
}

/// Picks between the offline and Microsoft providers based on `AppSettings::active_auth_mode`.
pub struct CompositeAuthenticationProvider {
    settings_repo: Arc<dyn SettingsRepository>,
    offline: Arc<dyn AuthenticationProviderPort>,
    microsoft: Arc<dyn AuthenticationProviderPort>,
}

impl CompositeAuthenticationProvider {
    pub fn new(
        settings_repo: Arc<dyn SettingsRepository>,
        offline: Arc<dyn AuthenticationProviderPort>,
        microsoft: Arc<dyn AuthenticationProviderPort>,
    ) -> Self {
        Self { settings_repo, offline, microsoft }
    }
}

#[async_trait]
impl AuthenticationProviderPort for CompositeAuthenticationProvider {
    async fn get_active_account(&self) -> Result<MinecraftAccount, LauncherError> {
        let settings = self.settings_repo.get().await?;
        match settings.active_auth_mode {
            AccountType::Offline => self.offline.get_active_account().await,
            AccountType::Microsoft => self.microsoft.get_active_account().await,
        }
    }
}
