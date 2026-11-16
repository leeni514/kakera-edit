use crate::models::{VisualNovel, format_playtime};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone};
use dioxus::prelude::*;
use std::cmp::Reverse;
use std::collections::HashMap;
///one colour per game, given out in this order to the most played games
const GAME_COLORS: [&str; 8] = [
    "#3987e5", "#d95926", "#199e70", "#c98500", "#d55181", "#008300", "#9085e9", "#e66767",
];
///shared by every game past the first eight
const OTHER_GAME_COLOR: &str = "#6f7f96";
///(days, label) for the range buttons
const RANGE_OPTIONS: [(i64, &str); 3] = [(7, "7 days"), (30, "30 days"), (90, "90 days")];
const CHART_HEIGHT_PX: u32 = 260;
///blue ramp for the activity calendar, from a little playtime to the most
const ACTIVITY_LEVEL_COLORS: [&str; 4] = ["#184f95", "#256abf", "#3987e5", "#86b6ef"];
///how many weeks the activity calendar shows, like github's year
const ACTIVITY_WEEKS: i64 = 53;
///one game's playtime on one day
#[derive(Clone, PartialEq)]
struct GamePlaytime {
    vn_id: u64,
    title: String,
    color: &'static str,
    seconds: u64,
}
#[derive(Clone, PartialEq)]
struct DayPlaytime {
    date: NaiveDate,
    games: Vec<GamePlaytime>,
    total_seconds: u64,
}
#[component]
pub fn PlaytimeView(vns: Vec<VisualNovel>) -> Element {
    let mut range_days = use_signal(|| 30_i64);
    let selected_range_days = *range_days.read();
    let today = Local::now().date_naive();
    let first_day = today - Duration::days(selected_range_days - 1);
    let colors = game_colors(&vns);
    let color_for = |vn_id: u64| colors.get(&vn_id).copied().unwrap_or(OTHER_GAME_COLOR);
    //seconds per (day, game), with sessions that cross midnight split between both days
    let mut daily_seconds: HashMap<(NaiveDate, u64), u64> = HashMap::new();
    let mut sessions_in_range: HashMap<u64, u32> = HashMap::new();
    for vn in &vns {
        for session in &vn.play_sessions {
            let Ok(started_at) = DateTime::parse_from_rfc3339(&session.started_at) else {
                continue;
            };
            let started_at = started_at.with_timezone(&Local);
            add_session_by_day(&mut daily_seconds, vn.id, started_at, session.duration_seconds);
            let start_date = started_at.date_naive();
            if start_date >= first_day && start_date <= today {
                *sessions_in_range.entry(vn.id).or_default() += 1;
            }
        }
    }
    let day_playtime = |date: NaiveDate| {
        let mut games: Vec<GamePlaytime> = vns
            .iter()
            .filter_map(|vn| {
                let seconds = *daily_seconds.get(&(date, vn.id))?;
                (seconds > 0).then(|| GamePlaytime {
                    vn_id: vn.id,
                    title: vn.title.clone(),
                    color: color_for(vn.id),
                    seconds,
                })
            })
            .collect();
        //stack in colour order so a game sits in the same spot on every bar
        games.sort_by_key(|game| color_slot(game.color));
        let total_seconds = games.iter().map(|game| game.seconds).sum();
        DayPlaytime { date, games, total_seconds }
    };
    let days: Vec<DayPlaytime> = (0..selected_range_days)
        .map(|offset| day_playtime(first_day + Duration::days(offset)))
        .collect();
    //the activity calendar always covers the last year, in sunday-first week columns ending with this week
    let activity_start = today
        - Duration::weeks(ACTIVITY_WEEKS - 1)
        - Duration::days(today.weekday().num_days_from_sunday() as i64);
    let activity_days: Vec<DayPlaytime> = (0..ACTIVITY_WEEKS * 7)
        .map(|offset| activity_start + Duration::days(offset))
        .take_while(|date| *date <= today)
        .map(day_playtime)
        .collect();
    let activity_total_seconds: u64 = activity_days.iter().map(|day| day.total_seconds).sum();
    let activity_total_text = format_playtime(activity_total_seconds);
    let activity_max_seconds = activity_days.iter().map(|day| day.total_seconds).max().unwrap_or(0);
    let month_labels = activity_month_labels(activity_start);
    let mut game_totals: Vec<GamePlaytime> = Vec::new();
    for game in days.iter().flat_map(|day| day.games.iter()) {
        match game_totals.iter_mut().find(|total| total.vn_id == game.vn_id) {
            Some(total) => total.seconds += game.seconds,
            None => game_totals.push(game.clone()),
        }
    }
    game_totals.sort_by_key(|game| Reverse(game.seconds));
    let range_total_seconds: u64 = days.iter().map(|day| day.total_seconds).sum();
    let days_played = days.iter().filter(|day| day.total_seconds > 0).count();
    let daily_average_text = format_playtime(range_total_seconds / selected_range_days as u64);
    let range_total_text = format_playtime(range_total_seconds);
    let max_day_seconds = days.iter().map(|day| day.total_seconds).max().unwrap_or(0);
    let step_seconds = axis_step_minutes(max_day_seconds) * 60;
    let axis_max_seconds = max_day_seconds.div_ceil(step_seconds).max(1) * step_seconds;
    let ticks: Vec<u64> = (0..=axis_max_seconds / step_seconds)
        .map(|index| index * step_seconds)
        .collect();
    let label_every = match selected_range_days {
        7 => 1,
        30 => 5,
        _ => 15,
    };
    let day_count = days.len();
    let shows_other_in_legend = game_totals.iter().any(|game| game.color == OTHER_GAME_COLOR);
    let legend_games: Vec<GamePlaytime> = game_totals
        .iter()
        .filter(|game| game.color != OTHER_GAME_COLOR)
        .cloned()
        .collect();
    rsx! {
        section { class: "settings-panel playtime-panel",
            div { class: "playtime-heading",
                h2 { "Playtime" }
                div { class: "topbar-segmented-control",
                    for (days_option , label) in RANGE_OPTIONS {
                        button {
                            key: "{days_option}",
                            class: if selected_range_days == days_option { "topbar-segment active" } else { "topbar-segment" },
                            onclick: move |_| {
                                range_days.set(days_option);
                            },
                            "{label}"
                        }
                    }
                }
            }
            div { class: "playtime-stats",
                div {
                    span { class: "stat-label", "Total" }
                    strong { "{range_total_text}" }
                }
                div {
                    span { class: "stat-label", "Daily average" }
                    strong { "{daily_average_text}" }
                }
                div {
                    span { class: "stat-label", "Days played" }
                    strong { "{days_played} of {selected_range_days}" }
                }
            }
            div { class: "settings-section",
                h3 { "Playtime per day" }
                if range_total_seconds == 0 {
                    p { class: "setting-help", "No playtime recorded in the last {selected_range_days} days." }
                } else {
                    div { class: "playtime-legend",
                        for game in legend_games {
                            span { key: "{game.vn_id}", class: "playtime-legend-item",
                                span {
                                    class: "playtime-swatch",
                                    style: "background: {game.color};",
                                }
                                "{game.title}"
                            }
                        }
                        if shows_other_in_legend {
                            span { class: "playtime-legend-item",
                                span {
                                    class: "playtime-swatch",
                                    style: "background: {OTHER_GAME_COLOR};",
                                }
                                "Other"
                            }
                        }
                    }
                    div {
                        class: "playtime-chart",
                        style: "--chart-height: {CHART_HEIGHT_PX}px;",
                        div { class: "playtime-y-axis",
                            for tick in ticks.clone() {
                                span {
                                    key: "{tick}",
                                    class: "playtime-tick-label",
                                    style: "bottom: {tick as f64 / axis_max_seconds as f64 * 100.0}%;",
                                    "{format_axis_label(tick)}"
                                }
                            }
                        }
                        div { class: "playtime-plot",
                            for tick in ticks {
                                div {
                                    key: "grid-{tick}",
                                    class: "playtime-gridline",
                                    style: "bottom: {tick as f64 / axis_max_seconds as f64 * 100.0}%;",
                                }
                            }
                            for (index , day) in days.iter().cloned().enumerate() {
                                {
                                    //keep tooltips near the chart edges from running off the panel
                                    let tooltip_class = if index < day_count / 4 {
                                        "playtime-tooltip align-start"
                                    } else if index >= day_count * 3 / 4 {
                                        "playtime-tooltip align-end"
                                    } else {
                                        "playtime-tooltip"
                                    };
                                    rsx! {
                                        div { key: "{day.date}", class: "playtime-day",
                                            div { class: "playtime-bar",
                                                for game in day.games.iter() {
                                                    div {
                                                        key: "{game.vn_id}",
                                                        class: "playtime-segment",
                                                        style: "height: {game.seconds as f64 / axis_max_seconds as f64 * 100.0}%; background: {game.color};",
                                                    }
                                                }
                                            }
                                            DayTooltip { class: tooltip_class, day: day.clone() }
                                        }
                                    }
                                }
                            }
                        }
                        div {}
                        div { class: "playtime-x-axis",
                            for (index , day) in days.iter().enumerate() {
                                span { key: "{day.date}", class: "playtime-x-label",
                                    //label every few days, counting back from today so today always has one
                                    if (day_count - 1 - index).is_multiple_of(label_every) {
                                        "{day.date.format(\"%b %-d\")}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "settings-section",
                h3 { "Games played" }
                if game_totals.is_empty() {
                    p { class: "setting-help", "No games played in the last {selected_range_days} days." }
                } else {
                    table { class: "playtime-table",
                        thead {
                            tr {
                                th { "Game" }
                                th { class: "numeric", "Playtime" }
                                th { class: "numeric", "Sessions" }
                                th { class: "numeric", "Share" }
                            }
                        }
                        tbody {
                            for game in game_totals {
                                tr { key: "{game.vn_id}",
                                    td {
                                        span { class: "playtime-table-game",
                                            span {
                                                class: "playtime-swatch",
                                                style: "background: {game.color};",
                                            }
                                            "{game.title}"
                                        }
                                    }
                                    td { class: "numeric", "{format_playtime(game.seconds)}" }
                                    td { class: "numeric",
                                        "{sessions_in_range.get(&game.vn_id).copied().unwrap_or(0)}"
                                    }
                                    td { class: "numeric",
                                        "{game.seconds * 100 / range_total_seconds}%"
                                    }
                                }
                            }
                        }
                    }
                    p { class: "setting-help",
                        "Sessions that cross midnight count toward both days. Playtime from before Kakera isn't included, since it has no dates."
                    }
                }
            }
            div { class: "settings-section",
                div { class: "activity-heading",
                    h3 { "Activity" }
                    span { class: "setting-help", "{activity_total_text} in the last year" }
                }
                div { class: "activity-calendar",
                    div { class: "activity-months",
                        for (column , label) in month_labels {
                            span {
                                key: "{column}",
                                class: "activity-month",
                                style: "grid-column: {column + 1};",
                                "{label}"
                            }
                        }
                    }
                    div { class: "activity-weekdays",
                        span { "" }
                        span { "Mon" }
                        span { "" }
                        span { "Wed" }
                        span { "" }
                        span { "Fri" }
                        span { "" }
                    }
                    div { class: "activity-grid",
                        for (index , day) in activity_days.into_iter().enumerate() {
                            {
                                let column = index as i64 / 7;
                                let row = index % 7;
                                //open tooltips downward on the top rows and inward near the edges
                                let vertical_class = if row < 3 { "below" } else { "above" };
                                let horizontal_class = if column < ACTIVITY_WEEKS / 4 {
                                    " align-start"
                                } else if column >= ACTIVITY_WEEKS * 3 / 4 {
                                    " align-end"
                                } else {
                                    ""
                                };
                                let cell_color = activity_color(day.total_seconds, activity_max_seconds);
                                rsx! {
                                    div { key: "{day.date}", class: "activity-cell",
                                        div {
                                            class: "activity-square",
                                            style: "background: {cell_color};",
                                        }
                                        DayTooltip {
                                            class: "playtime-tooltip {vertical_class}{horizontal_class}",
                                            day,
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "activity-key",
                        span { "Less" }
                        span {
                            class: "activity-square",
                            style: "background: {activity_color(0, 1)};",
                        }
                        for color in ACTIVITY_LEVEL_COLORS {
                            span {
                                key: "{color}",
                                class: "activity-square",
                                style: "background: {color};",
                            }
                        }
                        span { "More" }
                    }
                }
            }
        }
    }
}
///a day's date, total, and each game's time, shown when hovering a day
#[component]
fn DayTooltip(class: String, day: DayPlaytime) -> Element {
    let mut games_by_time = day.games.clone();
    games_by_time.sort_by_key(|game| Reverse(game.seconds));
    rsx! {
        div { class: "{class}",
            div { class: "playtime-tooltip-date", "{day.date.format(\"%a, %b %-d\")}" }
            if day.total_seconds == 0 {
                div { class: "playtime-tooltip-total", "No playtime" }
            } else {
                div { class: "playtime-tooltip-total", "{format_playtime(day.total_seconds)}" }
                for game in games_by_time {
                    div { key: "{game.vn_id}", class: "playtime-tooltip-row",
                        span {
                            class: "playtime-swatch",
                            style: "background: {game.color};",
                        }
                        span { class: "playtime-tooltip-title", "{game.title}" }
                        span { class: "playtime-tooltip-time", "{format_playtime(game.seconds)}" }
                    }
                }
            }
        }
    }
}
///empty days get the panel colour, the rest step up the blue ramp by share of the busiest day
fn activity_color(seconds: u64, max_seconds: u64) -> &'static str {
    if seconds == 0 || max_seconds == 0 {
        return "var(--bg-input)";
    }
    let level = ((seconds * 4).div_ceil(max_seconds) as usize).clamp(1, 4);
    ACTIVITY_LEVEL_COLORS[level - 1]
}
///(week column, month name) for each column that starts a new month, like github's top row
fn activity_month_labels(activity_start: NaiveDate) -> Vec<(i64, String)> {
    let mut labels: Vec<(i64, String)> = Vec::new();
    let mut previous_month = None;
    for column in 0..ACTIVITY_WEEKS {
        let week_start = activity_start + Duration::weeks(column);
        if previous_month != Some(week_start.month()) {
            previous_month = Some(week_start.month());
            labels.push((column, week_start.format("%b").to_string()));
        }
    }
    //the first column is usually mid-month, so drop its label if the next one would crowd it
    if labels.len() > 1 && labels[1].0 < 3 {
        labels.remove(0);
    }
    labels
}
///gives the most played games (all time) their own colour, so colours stay put when the range changes
fn game_colors(vns: &[VisualNovel]) -> HashMap<u64, &'static str> {
    let mut played: Vec<(u64, u64)> = vns
        .iter()
        .map(|vn| {
            let session_seconds = vn.play_sessions.iter().map(|session| session.duration_seconds).sum();
            (vn.id, session_seconds)
        })
        .filter(|(_, seconds)| *seconds > 0)
        .collect();
    played.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    played
        .into_iter()
        .zip(GAME_COLORS)
        .map(|((vn_id, _), color)| (vn_id, color))
        .collect()
}
fn color_slot(color: &str) -> usize {
    GAME_COLORS
        .iter()
        .position(|slot| *slot == color)
        .unwrap_or(GAME_COLORS.len())
}
///adds a session's seconds to each local day it covers
fn add_session_by_day(
    daily_seconds: &mut HashMap<(NaiveDate, u64), u64>,
    vn_id: u64,
    started_at: DateTime<Local>,
    duration_seconds: u64,
) {
    let ended_at = started_at + Duration::seconds(duration_seconds as i64);
    let mut cursor = started_at;
    while cursor < ended_at {
        let date = cursor.date_naive();
        let next_midnight = date
            .succ_opt()
            .and_then(|next_day| next_day.and_hms_opt(0, 0, 0))
            .and_then(|midnight| Local.from_local_datetime(&midnight).earliest())
            .filter(|midnight| *midnight > cursor)
            .unwrap_or(ended_at);
        let chunk_end = next_midnight.min(ended_at);
        *daily_seconds.entry((date, vn_id)).or_default() +=
            (chunk_end - cursor).num_seconds().max(0) as u64;
        cursor = chunk_end;
    }
}
///the smallest round step that fits the busiest day in four gridlines
fn axis_step_minutes(max_day_seconds: u64) -> u64 {
    let max_minutes = max_day_seconds.div_ceil(60).max(1);
    [15, 30, 60, 120, 180, 240, 360]
        .into_iter()
        .find(|step| max_minutes <= step * 4)
        .unwrap_or(360)
}
///"30m", "2h", "1h 30m"
fn format_axis_label(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = seconds % 3600 / 60;
    match (hours, minutes) {
        (0, minutes) => format!("{minutes}m"),
        (hours, 0) => format!("{hours}h"),
        (hours, minutes) => format!("{hours}h {minutes}m"),
    }
}
