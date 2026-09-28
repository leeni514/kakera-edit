use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
///the way kakera should launch a vn
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LaunchMode {
    Native,
    Wine,
    Proton,
    Steam,
}
///A visual novel stored in the library
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualNovel {
    pub id: u64,
    pub title: String,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub vndb_id: Option<String>,
    #[serde(default)]
    pub is_favourite: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub cover_path: Option<String>,
    #[serde(default)]
    pub executable_path: Option<String>,
    #[serde(default)]
    pub game_folder: Option<String>,
    #[serde(default)]
    pub launch_mode: LaunchMode,
    #[serde(default)]
    pub steam_app_id: Option<u32>,
    #[serde(default)]
    pub wine_binary: Option<String>,
    #[serde(default)]
    pub wine_prefix: Option<String>,
    #[serde(default)]
    pub wine_locale: Option<String>,
    #[serde(default)]
    pub proton_path: Option<String>,
    #[serde(default = "default_umu_game_id")]
    pub umu_game_id: String,
    #[serde(default)]
    pub launch_arguments: String,
    #[serde(default)]
    pub launch_environment: String,
    pub notes: String,
    pub routes: Vec<StoryRoute>,
    #[serde(default)]
    pub active_route: Option<String>,
    pub play_sessions: Vec<PlaySession>,
    ///playtime from before the vn was added to kakera, set by the user
    #[serde(default)]
    pub previous_playtime_seconds: u64,
}
impl VisualNovel {
    ///a new vn with nothing set except its id and title
    pub fn new(id: u64, title: String) -> Self {
        Self {
            id,
            title,
            cover_url: None,
            description: None,
            vndb_id: None,
            is_favourite: false,
            tags: Vec::new(),
            cover_path: None,
            executable_path: None,
            game_folder: None,
            launch_mode: LaunchMode::default(),
            steam_app_id: None,
            wine_binary: None,
            wine_prefix: None,
            wine_locale: None,
            proton_path: None,
            umu_game_id: default_umu_game_id(),
            launch_arguments: String::new(),
            launch_environment: String::new(),
            notes: String::new(),
            routes: Vec::new(),
            active_route: None,
            play_sessions: Vec::new(),
            previous_playtime_seconds: 0,
        }
    }
    ///recorded sessions plus any playtime from before kakera
    pub fn total_playtime_seconds(&self) -> u64 {
        self.previous_playtime_seconds
            + self
                .play_sessions
                .iter()
                .map(|session| session.duration_seconds)
                .sum::<u64>()
    }
}
impl Default for LaunchMode {
    fn default() -> Self {
        LaunchMode::Native
    }
}
pub fn default_umu_game_id() -> String {
    "umu-default".to_string()
}
///A vn route
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoryRoute {
    pub name: String,
    pub completed: bool,
    pub notes: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaySession {
    #[serde(alias = "visual_novel_id")]
    pub vn_id: u64,
    pub started_at: String,
    pub duration_seconds: u64,
    pub notes: Option<String>,
}
///types of notificiations
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AppNotification {
    pub level: NotificationLevel,
    pub message: String,
}
///the order to display vns in the library
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LibrarySortMode {
    Added,
    TitleAsc,
    TitleDesc,
    LastPlayed,
    MostPlaytime,
}
impl Default for LibrarySortMode {
    fn default() -> Self {
        LibrarySortMode::Added
    }
}
///the "categories" i guess? of vns that u can set in the library.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LibraryFilterMode {
    All,
    Favourites,
}
impl Default for LibraryFilterMode {
    fn default() -> Self {
        LibraryFilterMode::All
    }
}
///settings set in the settings panel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    #[serde(default)]
    pub discord_rich_presence_enabled: bool,
    #[serde(default = "default_discord_status_text")]
    pub discord_status_text: String,
    #[serde(default)]
    pub discord_show_active_route: bool,
    #[serde(default)]
    pub discord_custom_cover_url: String,
    #[serde(default = "default_discord_idle_name")]
    pub discord_idle_name: String,
    #[serde(default)]
    pub discord_idle_image_url: String,
    #[serde(default)]
    pub vn_library_folder: Option<String>,
    #[serde(default)]
    pub library_sort_mode: LibrarySortMode,
    #[serde(default)]
    pub library_filter_mode: LibraryFilterMode,
}
fn default_discord_status_text() -> String {
    "Reading".to_string()
}
fn default_discord_idle_name() -> String {
    "Kakera".to_string()
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            discord_rich_presence_enabled: true,
            discord_status_text: default_discord_status_text(),
            discord_show_active_route: true,
            discord_custom_cover_url: String::new(),
            discord_idle_name: default_discord_idle_name(),
            discord_idle_image_url: String::new(),
            vn_library_folder: None,
            library_sort_mode: LibrarySortMode::default(),
            library_filter_mode: LibraryFilterMode::default(),
        }
    }
}
