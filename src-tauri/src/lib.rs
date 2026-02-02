use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconId},
    AppHandle, Emitter, Manager, PhysicalPosition, ActivationPolicy,
};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UsageLimit {
    pub utilization: Option<f64>,
    pub resets_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ExtraUsage {
    pub is_enabled: Option<bool>,
    pub monthly_limit: Option<i64>,
    pub used_credits: Option<i64>,
    pub utilization: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UsageResponse {
    pub five_hour: Option<UsageLimit>,
    pub seven_day: Option<UsageLimit>,
    pub seven_day_sonnet: Option<UsageLimit>,
    pub seven_day_opus: Option<UsageLimit>,
    pub extra_usage: Option<ExtraUsage>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ClaudeCredentials {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OAuthToken>,
}

// Synchronous version for reading raw credentials JSON
fn get_raw_credentials_json() -> Result<String, String> {
    use std::process::Command;
    let output = Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .map_err(|e| format!("Failed to run security command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Keychain access failed: {}", stderr));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in credentials: {}", e))
        .map(|s| s.trim().to_string())
}

fn update_keychain_credentials(new_token: &OAuthToken) -> Result<(), String> {
    use std::process::Command;

    // Read existing credentials to preserve other fields
    let json_str = get_raw_credentials_json()?;
    let mut creds: ClaudeCredentialsFull = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse credentials: {}", e))?;

    // Update the OAuth token
    creds.claude_ai_oauth = Some(new_token.clone());

    let new_json = serde_json::to_string(&creds)
        .map_err(|e| format!("Failed to serialize credentials: {}", e))?;

    // Delete existing keychain entry
    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", "Claude Code-credentials"])
        .output();

    // Add updated credentials
    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-s", "Claude Code-credentials",
            "-a", "",
            "-w", &new_json,
            "-U",
        ])
        .output()
        .map_err(|e| format!("Failed to update keychain: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Keychain update failed: {}", stderr));
    }

    Ok(())
}

async fn refresh_oauth_token(refresh_token: &str) -> Result<OAuthToken, String> {
    let client = reqwest::Client::new();

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let response = client
        .post("https://console.anthropic.com/v1/oauth/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed ({}): {}", status, body));
    }

    let token_response: TokenRefreshResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    // Read current credentials to preserve subscription info
    let json_str = get_raw_credentials_json()?;
    let current_creds: ClaudeCredentials = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse credentials: {}", e))?;

    let current_oauth = current_creds.claude_ai_oauth.unwrap_or(OAuthToken {
        access_token: String::new(),
        refresh_token: None,
        expires_at: None,
        scopes: None,
        subscription_type: None,
        rate_limit_tier: None,
    });

    let new_expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64 + (token_response.expires_in * 1000))
        .ok();

    let new_token = OAuthToken {
        access_token: token_response.access_token,
        refresh_token: Some(token_response.refresh_token),
        expires_at: new_expires_at,
        scopes: current_oauth.scopes,
        subscription_type: current_oauth.subscription_type,
        rate_limit_tier: current_oauth.rate_limit_tier,
    };

    // Update keychain with new credentials
    update_keychain_credentials(&new_token)?;

    Ok(new_token)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OAuthToken {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<i64>,
    #[serde(rename = "scopes")]
    scopes: Option<Vec<String>>,
    #[serde(rename = "subscriptionType")]
    subscription_type: Option<String>,
    #[serde(rename = "rateLimitTier")]
    rate_limit_tier: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ClaudeCredentialsFull {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OAuthToken>,
}

#[derive(Debug, Deserialize)]
struct TokenRefreshResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AccountInfo {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub full_name: Option<String>,
    pub subscription: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProfileResponse {
    account: ProfileAccount,
}

#[derive(Debug, Deserialize)]
struct ProfileAccount {
    email: String,
    display_name: Option<String>,
    full_name: Option<String>,
}

pub struct AppState {
    pub usage: Arc<Mutex<Option<UsageResponse>>>,
    pub last_error: Arc<Mutex<Option<String>>>,
    pub account: Arc<Mutex<Option<AccountInfo>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            usage: Arc::new(Mutex::new(None)),
            last_error: Arc::new(Mutex::new(None)),
            account: Arc::new(Mutex::new(None)),
        }
    }
}

struct CredentialsInfo {
    access_token: String,
    refresh_token: Option<String>,
    subscription: Option<String>,
}

fn format_subscription(subscription_type: Option<&str>, rate_limit_tier: Option<&str>) -> Option<String> {
    match subscription_type {
        Some("max") => {
            let multiplier = rate_limit_tier
                .and_then(|tier| {
                    if tier.contains("20x") { Some("20x") }
                    else if tier.contains("5x") { Some("5x") }
                    else { None }
                })
                .unwrap_or("");
            Some(format!("Max {}", multiplier).trim().to_string())
        }
        Some("pro") => Some("Pro".to_string()),
        Some("free") | None => Some("Free".to_string()),
        Some(other) => Some(other.to_string()),
    }
}

fn get_claude_credentials() -> Result<CredentialsInfo, String> {
    let json_str = get_raw_credentials_json()?;

    let creds: ClaudeCredentials =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse credentials: {}", e))?;

    creds
        .claude_ai_oauth
        .map(|oauth| CredentialsInfo {
            access_token: oauth.access_token,
            refresh_token: oauth.refresh_token,
            subscription: format_subscription(
                oauth.subscription_type.as_deref(),
                oauth.rate_limit_tier.as_deref(),
            ),
        })
        .ok_or_else(|| "No OAuth token found in credentials".to_string())
}

fn get_claude_token() -> Result<String, String> {
    get_claude_credentials().map(|c| c.access_token)
}

fn get_refresh_token() -> Result<String, String> {
    get_claude_credentials()
        .and_then(|c| c.refresh_token.ok_or_else(|| "No refresh token found".to_string()))
}

async fn fetch_usage_internal(token: &str) -> Result<UsageResponse, (String, bool)> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", format!("Bearer {}", token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
        .map_err(|e| (format!("Request failed: {}", e), false))?;

    let status = response.status();
    if !status.is_success() {
        let is_auth_error = status.as_u16() == 401;
        return Err((format!("API returned status: {}", status), is_auth_error));
    }

    response
        .json::<UsageResponse>()
        .await
        .map_err(|e| (format!("Failed to parse response: {}", e), false))
}

async fn fetch_usage(token: &str) -> Result<UsageResponse, String> {
    match fetch_usage_internal(token).await {
        Ok(usage) => Ok(usage),
        Err((err, is_auth_error)) => {
            if is_auth_error {
                // Try to refresh the token
                if let Ok(refresh_token) = get_refresh_token() {
                    if let Ok(new_token) = refresh_oauth_token(&refresh_token).await {
                        // Retry with new token
                        return fetch_usage_internal(&new_token.access_token)
                            .await
                            .map_err(|(e, _)| e);
                    }
                }
            }
            Err(err)
        }
    }
}

async fn fetch_profile_internal(token: &str) -> Result<AccountInfo, (String, bool)> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.anthropic.com/api/oauth/profile")
        .header("Authorization", format!("Bearer {}", token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
        .map_err(|e| (format!("Request failed: {}", e), false))?;

    let status = response.status();
    if !status.is_success() {
        let is_auth_error = status.as_u16() == 401;
        return Err((format!("API returned status: {}", status), is_auth_error));
    }

    let profile: ProfileResponse = response
        .json()
        .await
        .map_err(|e| (format!("Failed to parse profile: {}", e), false))?;

    Ok(AccountInfo {
        email: Some(profile.account.email),
        display_name: profile.account.display_name,
        full_name: profile.account.full_name,
        subscription: None, // Set by caller from credentials
    })
}

async fn fetch_profile(token: &str) -> Result<AccountInfo, String> {
    match fetch_profile_internal(token).await {
        Ok(profile) => Ok(profile),
        Err((err, is_auth_error)) => {
            if is_auth_error {
                // Try to refresh the token
                if let Ok(refresh_token) = get_refresh_token() {
                    if let Ok(new_token) = refresh_oauth_token(&refresh_token).await {
                        // Retry with new token
                        return fetch_profile_internal(&new_token.access_token)
                            .await
                            .map_err(|(e, _)| e);
                    }
                }
            }
            Err(err)
        }
    }
}

fn auto_select_chrome_profile(email: &str) -> Option<String> {
    let profiles = get_chrome_profiles();
    for profile in profiles {
        if let Some(ref profile_email) = profile.email {
            if profile_email.eq_ignore_ascii_case(email) {
                // Save the matched profile
                let home = std::env::var("HOME").unwrap_or_default();
                let config_path = format!("{}/.claude-usage-monitor-profile", home);
                let _ = std::fs::write(&config_path, &profile.id);
                return Some(profile.id);
            }
        }
    }
    None
}

fn update_tray_title(app: &AppHandle, usage: &UsageResponse) {
    if let Some(tray) = app.tray_by_id(&TrayIconId::new("main")) {
        let title = if let Some(ref five_hour) = usage.five_hour {
            format!("{}%", five_hour.utilization.unwrap_or(0.0) as i32)
        } else {
            "--%".to_string()
        };
        let _ = tray.set_title(Some(&title));
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChromeProfile {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
}

#[tauri::command]
fn get_chrome_profiles() -> Vec<ChromeProfile> {
    let mut profiles = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();
    let chrome_path = format!("{}/Library/Application Support/Google/Chrome", home);

    if let Ok(entries) = std::fs::read_dir(&chrome_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "Default" || name.starts_with("Profile ") {
                let prefs_path = format!("{}/{}/Preferences", chrome_path, name);
                if let Ok(content) = std::fs::read_to_string(&prefs_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        let profile_name = json["profile"]["name"]
                            .as_str()
                            .unwrap_or(&name)
                            .to_string();
                        let email = json["account_info"]
                            .as_array()
                            .and_then(|arr| arr.first())
                            .and_then(|acc| acc["email"].as_str())
                            .map(|s| s.to_string());

                        profiles.push(ChromeProfile {
                            id: name,
                            name: profile_name,
                            email,
                        });
                    }
                }
            }
        }
    }
    profiles
}

#[tauri::command]
fn get_selected_profile() -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let config_path = format!("{}/.claude-usage-monitor-profile", home);
    std::fs::read_to_string(&config_path).ok()
}

#[tauri::command]
fn set_selected_profile(profile_id: String) {
    let home = std::env::var("HOME").unwrap_or_default();
    let config_path = format!("{}/.claude-usage-monitor-profile", home);
    let _ = std::fs::write(&config_path, &profile_id);
}

#[tauri::command]
fn open_url(url: String) {
    let home = std::env::var("HOME").unwrap_or_default();
    let config_path = format!("{}/.claude-usage-monitor-profile", home);
    let profile = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|_| "Default".to_string());

    // Use Chrome binary directly to ensure profile is respected even when Chrome is running
    let _ = std::process::Command::new("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
        .args([&format!("--profile-directory={}", profile), &url])
        .spawn();
}

#[tauri::command]
async fn get_usage(state: tauri::State<'_, AppState>) -> Result<Option<UsageResponse>, String> {
    let usage = state.usage.lock().await;
    Ok(usage.clone())
}

#[tauri::command]
async fn get_account(state: tauri::State<'_, AppState>) -> Result<Option<AccountInfo>, String> {
    let account = state.account.lock().await;
    Ok(account.clone())
}

#[tauri::command]
async fn set_window_height(app: AppHandle, height: u32) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_size(tauri::LogicalSize::new(180.0, height as f64))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn get_last_error(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let error = state.last_error.lock().await;
    Ok(error.clone())
}

#[tauri::command]
async fn refresh_usage(state: tauri::State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let token = get_claude_token()?;
    match fetch_usage(&token).await {
        Ok(usage) => {
            update_tray_title(&app, &usage);
            *state.usage.lock().await = Some(usage.clone());
            *state.last_error.lock().await = None;
            let _ = app.emit("usage-updated", usage);
        }
        Err(e) => {
            *state.last_error.lock().await = Some(e.clone());
            let _ = app.emit("usage-error", e.clone());
            return Err(e);
        }
    }
    Ok(())
}

async fn start_polling(app: AppHandle, state: Arc<Mutex<Option<UsageResponse>>>, error_state: Arc<Mutex<Option<String>>>) {
    let mut ticker = interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        match get_claude_token() {
            Ok(token) => match fetch_usage(&token).await {
                Ok(usage) => {
                    update_tray_title(&app, &usage);
                    *state.lock().await = Some(usage.clone());
                    *error_state.lock().await = None;
                    let _ = app.emit("usage-updated", usage);
                }
                Err(e) => {
                    *error_state.lock().await = Some(e.clone());
                    let _ = app.emit("usage-error", e);
                }
            },
            Err(e) => {
                *error_state.lock().await = Some(e.clone());
                let _ = app.emit("usage-error", e);
            }
        }
    }
}

fn toggle_window(app: &AppHandle, tray_x: f64, tray_y: f64) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            // Position window below tray icon
            let window_width = 180.0;
            let x = (tray_x - window_width / 2.0) as i32;
            let y = tray_y as i32; // Right below the menu bar
            let _ = window.set_position(PhysicalPosition::new(x, y));
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::default();
    let usage_state = app_state.usage.clone();
    let error_state = app_state.last_error.clone();
    let account_state = app_state.account.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .setup(move |app| {
            // Hide from dock
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).item(&quit).build()?;

            let icon_bytes = include_bytes!("../icons/32x32.png");
            let icon_image = image::load_from_memory(icon_bytes)
                .expect("Failed to load icon")
                .to_rgba8();
            let (width, height) = icon_image.dimensions();
            let icon = Image::new_owned(icon_image.into_raw(), width, height);

            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .menu(&menu)
                .menu_on_left_click(false)
                .title("--%")
                .tooltip("Claude Usage Monitor")
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        position,
                        ..
                    } = event {
                        toggle_window(tray.app_handle(), position.x, position.y + 22.0);
                    }
                })
                .build(app)?;

            // Initial fetch
            let app_handle = app.handle().clone();
            let usage_clone = usage_state.clone();
            let error_clone = error_state.clone();
            let account_clone = account_state.clone();

            tauri::async_runtime::spawn(async move {
                match get_claude_credentials() {
                    Ok(creds) => {
                        // Fetch profile and auto-select Chrome profile
                        if let Ok(mut account) = fetch_profile(&creds.access_token).await {
                            account.subscription = creds.subscription;
                            if let Some(ref email) = account.email {
                                auto_select_chrome_profile(email);
                            }
                            *account_clone.lock().await = Some(account.clone());
                            let _ = app_handle.emit("account-updated", account);
                        }

                        // Fetch usage
                        match fetch_usage(&creds.access_token).await {
                            Ok(usage) => {
                                update_tray_title(&app_handle, &usage);
                                *usage_clone.lock().await = Some(usage.clone());
                                let _ = app_handle.emit("usage-updated", usage);
                            }
                            Err(e) => {
                                *error_clone.lock().await = Some(e.clone());
                                let _ = app_handle.emit("usage-error", e);
                            }
                        }
                    },
                    Err(e) => {
                        *error_clone.lock().await = Some(e.clone());
                        let _ = app_handle.emit("usage-error", e);
                    }
                }
            });

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(start_polling(app_handle, usage_state, error_state));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_usage, get_last_error, refresh_usage, open_url, get_chrome_profiles, get_selected_profile, set_selected_profile, set_window_height, get_account])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
