use dioxus::prelude::*;
#[component]
pub fn SettingsView(
    discord_rich_presence_enabled: bool,
    discord_status_text: String,
    discord_show_active_route: bool,
    discord_custom_cover_url: String,
    data_dir_text: String,
    vn_library_folder: Option<String>,
    vn_scan_in_progress: bool,
    on_discord_rich_presence_change: EventHandler<bool>,
    on_discord_status_text_change: EventHandler<String>,
    on_discord_show_active_route_change: EventHandler<bool>,
    on_discord_custom_cover_url_change: EventHandler<String>,
    on_open_data_folder: EventHandler<()>,
    on_open_logs_folder: EventHandler<()>,
    on_choose_vn_folder: EventHandler<()>,
    on_scan_vn_folder: EventHandler<()>,
) -> Element {
    let mut discord_status_text_draft = use_signal(|| discord_status_text.clone());
    let mut discord_custom_cover_url_draft = use_signal(|| discord_custom_cover_url.clone());
    let discord_status_text_value = discord_status_text_draft.read().clone();
    let discord_custom_cover_url_value = discord_custom_cover_url_draft.read().clone();
    let vn_folder_is_set = vn_library_folder.is_some();
    let vn_folder_text = vn_library_folder.unwrap_or_else(|| "Not set".to_string());
    rsx! {
        section { class: "settings-panel",
            h2 { "Settings" }
            div { class: "settings-section",
                h3 { "Discord Rich Presence" }
                label { class: "setting-row",
                    span { "Enable Rich Presence" }
                    input {
                        class: "setting-checkbox",
                        r#type: "checkbox",
                        checked: discord_rich_presence_enabled,
                        onchange: move |event| {
                            on_discord_rich_presence_change.call(event.checked());
                        },
                    }
                }
                label { class: "setting-row",
                    span { "Default status text" }
                    input {
                        value: "{discord_status_text_value}",
                        oninput: move |event| {
                            discord_status_text_draft.set(event.value());
                        },
                        onblur: move |_| {
                            on_discord_status_text_change.call(discord_status_text_draft.read().clone());
                        },
                    }
                }
                label { class: "setting-row",
                    span { "Show active route" }
                    input {
                        class: "setting-checkbox",
                        r#type: "checkbox",
                        checked: discord_show_active_route,
                        onchange: move |event| {
                            on_discord_show_active_route_change.call(event.checked());
                        },
                    }
                }
                label { class: "setting-row",
                    span { "Custom cover URL" }
                    input {
                        value: "{discord_custom_cover_url_value}",
                        placeholder: "Leave blank to use VNDB cover",
                        oninput: move |event| {
                            discord_custom_cover_url_draft.set(event.value());
                        },
                        onblur: move |_| {
                            on_discord_custom_cover_url_change
                                .call(discord_custom_cover_url_draft.read().clone());
                        },
                    }
                }
                p { class: "setting-help", "Show the VN being played on your Discord profile." }
            }
            div { class: "settings-section",
                h3 { "VN Folder" }
                div { class: "setting-row",
                    span { "VN folder" }
                    code { class: "setting-path", "{vn_folder_text}" }
                }
                button {
                    class: "fp-button",
                    onclick: move |_| {
                        on_choose_vn_folder.call(());
                    },
                    "Choose VN folder"
                }
                button {
                    class: "fp-button",
                    disabled: !vn_folder_is_set || vn_scan_in_progress,
                    onclick: move |_| {
                        on_scan_vn_folder.call(());
                    },
                    if vn_scan_in_progress {
                        "Scanning..."
                    } else {
                        "Scan now"
                    }
                }
                p { class: "setting-help",
                    "Each folder inside the VN folder is added as a VN, with its game .exe and VNDB info filled in. Folders already in your library are skipped."
                }
            }
            div { class: "settings-section",
                h3 { "Data" }
                div { class: "setting-row",
                    span { "Data folder" }
                    code { class: "setting-path", "{data_dir_text}" }
                }
                button {
                    class: "fp-button",
                    onclick: move |_| {
                        on_open_data_folder.call(());
                    },
                    "Open data folder"
                }
                button {
                    class: "fp-button",
                    onclick: move |_| {
                        on_open_logs_folder.call(());
                    },
                    "Open logs folder"
                }
            }
        }
    }
}
