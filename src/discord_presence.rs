use crate::models::{AppSettings, VisualNovel};
use chrono::{DateTime, Utc};
use discord_rich_presence::error::Error as DiscordError;
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use std::sync::{Mutex, MutexGuard, OnceLock};
const DISCORD_APP_ID: &str = "1512156673358172312";
///the one discord connection kakera keeps open, shared by the idle and vn presence
struct PresenceState {
    client: Option<DiscordIpcClient>,
    running_vn_count: u32,
}
static PRESENCE: Mutex<PresenceState> = Mutex::new(PresenceState {
    client: None,
    running_vn_count: 0,
});
///when kakera was opened, used as the idle presence start time
static APP_STARTED_AT: OnceLock<DateTime<Utc>> = OnceLock::new();
///shows the idle presence (or clears it when rich presence is off), unless a vn is running
pub fn show_idle(settings: &AppSettings) {
    let mut state = lock_presence();
    if state.running_vn_count == 0 {
        update_activity(&mut state, settings, idle_activity(settings));
    }
}
///shows the currently launched vn
pub fn show_vn(vn: &VisualNovel, started_at: DateTime<Utc>, settings: &AppSettings) {
    let mut state = lock_presence();
    state.running_vn_count += 1;
    update_activity(&mut state, settings, vn_activity(vn, started_at, settings));
}
///goes back to the idle presence once the last running vn closes
pub fn vn_closed(settings: &AppSettings) {
    let mut state = lock_presence();
    state.running_vn_count = state.running_vn_count.saturating_sub(1);
    if state.running_vn_count == 0 {
        update_activity(&mut state, settings, idle_activity(settings));
    }
}
fn lock_presence() -> MutexGuard<'static, PresenceState> {
    PRESENCE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
fn idle_activity(settings: &AppSettings) -> activity::Activity<'static> {
    let started_at = APP_STARTED_AT.get_or_init(Utc::now);
    let name = if settings.discord_idle_name.trim().is_empty() {
        "Kakera".to_string()
    } else {
        settings.discord_idle_name.clone()
    };
    let mut assets = activity::Assets::new().large_text(name.clone());
    if !settings.discord_idle_image_url.is_empty() {
        assets = assets.large_image(settings.discord_idle_image_url.clone());
    }
    activity::Activity::new()
        .name(name)
        .timestamps(activity::Timestamps::new().start(started_at.timestamp_millis()))
        .assets(assets)
        .activity_type(activity::ActivityType::Playing)
}
fn vn_activity(
    vn: &VisualNovel,
    started_at: DateTime<Utc>,
    settings: &AppSettings,
) -> activity::Activity<'static> {
    let timestamps = activity::Timestamps::new().start(started_at.timestamp_millis());
    let mut assets = activity::Assets::new().large_text(vn.title.clone());
    let status_text = if settings.discord_show_active_route {
        match vn.active_route.clone() {
            Some(route_name) => format!("Reading the {route_name} route."),
            None => settings.discord_status_text.clone(),
        }
    } else {
        settings.discord_status_text.clone()
    };
    let cover_image = if !settings.discord_custom_cover_url.is_empty() {
        Some(settings.discord_custom_cover_url.clone())
    } else if vn.cover_url.is_some() {
        vn.cover_url.clone()
    } else if !settings.discord_idle_image_url.is_empty() {
        Some(settings.discord_idle_image_url.clone())
    } else {
        None
    };
    if let Some(cover_url) = cover_image {
        assets = assets.large_image(cover_url);
    }
    activity::Activity::new()
        .name(vn.title.clone())
        .details(status_text)
        .timestamps(timestamps)
        .assets(assets)
        .activity_type(activity::ActivityType::Playing)
}
///sends the activity to discord, or clears it when rich presence is turned off
fn update_activity(
    state: &mut PresenceState,
    settings: &AppSettings,
    activity: activity::Activity<'static>,
) {
    if !settings.discord_rich_presence_enabled {
        if let Some(mut client) = state.client.take() {
            let _ = client.clear_activity();
            let _ = client.close();
        }
        return;
    }
    //discord may have restarted since the last update, so reconnect once on failure
    let result = set_activity(state, activity.clone()).or_else(|_| {
        state.client = None;
        set_activity(state, activity)
    });
    if let Err(error) = result {
        state.client = None;
        println!("Could not update Discord Rich Presence: {error}");
    }
}
fn set_activity(
    state: &mut PresenceState,
    activity: activity::Activity<'static>,
) -> Result<(), DiscordError> {
    let client = match &mut state.client {
        Some(client) => client,
        None => {
            let mut client = DiscordIpcClient::new(DISCORD_APP_ID);
            client.connect()?;
            state.client.insert(client)
        }
    };
    client.set_activity(activity)?;
    match client.recv() {
        Ok((_opcode, response)) => {
            println!("Discord Rich Presence response: {response}");
        }
        Err(error) => {
            println!("Could not read Discord Rich Presence response: {error}");
        }
    }
    Ok(())
}
