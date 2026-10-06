use crate::vndb::{fetch_route_characters, fetch_vn_details};
use dioxus::prelude::*;
///length, rating, developer and tags fetched from vndb when the detail page opens
#[component]
pub fn VndbInfo(vndb_id: String) -> Element {
    let details = use_resource(move || fetch_vn_details(vndb_id.clone()));
    let content = match &*details.read() {
        None => rsx! { p { class: "vndb-info-status", "Loading VNDB info..." } },
        Some(Err(error)) => rsx! { p { class: "vndb-info-status", "Could not load VNDB info: {error}" } },
        Some(Ok(None)) => rsx! { p { class: "vndb-info-status", "VNDB has no entry with this ID." } },
        Some(Ok(Some(details))) => {
            let length_text = match details.length_minutes {
                Some(minutes) if minutes >= 60 => {
                    format!("~{}h ({} votes)", (minutes + 30) / 60, details.length_votes)
                }
                Some(minutes) => format!("~{minutes}m ({} votes)", details.length_votes),
                None => "Unknown".to_string(),
            };
            let rating_text = match details.rating {
                Some(rating) => format!("{:.2} ({} votes)", rating / 10.0, details.votecount),
                None => "No rating yet".to_string(),
            };
            let developers_text = details
                .developers
                .iter()
                .map(|developer| developer.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let released_text = details.released.clone().unwrap_or_else(|| "Unknown".to_string());
            let top_tags = details.top_tags(12);
            rsx! {
                dl { class: "vndb-info-grid",
                    dt { "Length" }
                    dd { "{length_text}" }
                    dt { "Rating" }
                    dd { "{rating_text}" }
                    if !developers_text.is_empty() {
                        dt { "Developer" }
                        dd { "{developers_text}" }
                    }
                    dt { "Released" }
                    dd { "{released_text}" }
                }
                if !top_tags.is_empty() {
                    div { class: "tag-chip-list vndb-tag-list",
                        for tag in top_tags {
                            span { class: "tag-chip", "{tag}" }
                        }
                    }
                }
            }
        }
    };
    rsx! {
        h3 { "VNDB" }
        div { class: "vndb-info", {content} }
    }
}
///lists the vn's main characters from vndb so their routes can be added in one go
#[component]
pub fn RouteSuggestions(
    vn_id: u64,
    vndb_id: String,
    existing_routes: Vec<String>,
    on_route_add: EventHandler<(u64, String)>,
) -> Element {
    let mut suggestions = use_signal(|| None::<Vec<String>>);
    let mut selected = use_signal(Vec::<String>::new);
    let mut status = use_signal(String::new);
    let status_text = status.read().clone();
    let is_new_route = |name: &String| {
        !existing_routes
            .iter()
            .any(|route| route.eq_ignore_ascii_case(name))
    };
    let new_suggestions: Option<Vec<String>> = suggestions
        .read()
        .as_ref()
        .map(|names| names.iter().filter(|name| is_new_route(name)).cloned().collect());
    rsx! {
        div { class: "route-suggestions",
            if let Some(names) = new_suggestions {
                if names.is_empty() {
                    p { class: "vndb-info-status", "All of VNDB's main characters are already routes." }
                } else {
                    p { class: "vndb-info-status", "Main characters on VNDB, tick the ones with a route:" }
                    div { class: "route-list",
                        for name in names.clone() {
                            {
                                let is_selected = selected.read().contains(&name);
                                rsx! {
                                    label { class: "route-suggestion",
                                        input {
                                            r#type: "checkbox",
                                            class: "route-checkbox",
                                            checked: is_selected,
                                            onchange: move |_| {
                                                let mut selected = selected.write();
                                                if let Some(index) = selected.iter().position(|selected_name| selected_name == &name) {
                                                    selected.remove(index);
                                                } else {
                                                    selected.push(name.clone());
                                                }
                                            },
                                        }
                                        span { class: "route-name", "{name}" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "route-suggestion-actions",
                        button {
                            disabled: selected.read().is_empty(),
                            onclick: move |_| {
                                for name in selected.read().iter() {
                                    on_route_add.call((vn_id, name.clone()));
                                }
                                selected.set(Vec::new());
                                suggestions.set(None);
                            },
                            "Add selected"
                        }
                        button {
                            class: "secondary-launch-button",
                            onclick: move |_| {
                                selected.set(Vec::new());
                                suggestions.set(None);
                            },
                            "Cancel"
                        }
                    }
                }
            } else {
                button {
                    class: "secondary-launch-button",
                    onclick: move |_| {
                        let vndb_id = vndb_id.clone();
                        status.set("Loading characters from VNDB...".to_string());
                        spawn(async move {
                            match fetch_route_characters(vndb_id).await {
                                Ok(names) => {
                                    status.set(String::new());
                                    selected.set(names.clone());
                                    suggestions.set(Some(names));
                                }
                                Err(error) => {
                                    status.set(format!("Could not load characters from VNDB: {error}"));
                                }
                            }
                        });
                    },
                    "Suggest routes from VNDB"
                }
                if !status_text.is_empty() {
                    p { class: "vndb-info-status", "{status_text}" }
                }
            }
        }
    }
}
