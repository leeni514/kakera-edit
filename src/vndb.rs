use serde::{Deserialize, Serialize};
use std::time::Duration;
///vn result returned from VNDB
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VndbSearchResult {
    pub id: String,
    pub title: String,
    pub image: Option<VndbImage>,
    pub description: Option<String>,
    pub released: Option<String>,
}
///vndb cover info
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VndbImage {
    pub url: Option<String>,
}
///vndb sends this back as a json
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct VndbSearchResponse {
    pub results: Vec<VndbSearchResult>,
}
///the json kakera will send to vndb
#[derive(Debug, Clone, Serialize)]
struct VndbSearchRequest {
    pub filters: serde_json::Value,
    pub fields: String,
    pub sort: String,
    pub results: u32,
}
///searches vndb for vns matching the query
pub async fn search_vns(query: String) -> Result<Vec<VndbSearchResult>, reqwest::Error> {
    let client = reqwest::Client::new();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let request_body = VndbSearchRequest {
        filters: serde_json::json!(["search", "=", query]),
        fields: "id, title, image.url, description, released".to_string(),
        sort: "searchrank".to_string(),
        results: 20,
    };
    let response = client
        .post("https://api.vndb.org/kana/vn")
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?
        .json::<VndbSearchResponse>()
        .await?;
    Ok(response.results)
}
///extra vn info shown on the detail page
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VndbDetails {
    pub released: Option<String>,
    ///average play time from user votes
    pub length_minutes: Option<u32>,
    pub length_votes: u32,
    ///bayesian rating from 10 to 100
    pub rating: Option<f64>,
    pub votecount: u32,
    pub developers: Vec<VndbDeveloper>,
    pub tags: Vec<VndbTag>,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VndbDeveloper {
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VndbTag {
    pub name: String,
    pub rating: f64,
    ///0 none, 1 minor, 2 major
    pub spoiler: u8,
    ///"cont" content, "ero" sexual content, "tech" technical
    pub category: String,
}
impl VndbDetails {
    ///highest rated tags, without spoilers or sexual content
    pub fn top_tags(&self, count: usize) -> Vec<String> {
        let mut tags: Vec<&VndbTag> = self
            .tags
            .iter()
            .filter(|tag| tag.spoiler == 0 && tag.category != "ero")
            .collect();
        tags.sort_by(|a, b| b.rating.total_cmp(&a.rating));
        tags.into_iter().take(count).map(|tag| tag.name.clone()).collect()
    }
}
#[derive(Debug, Clone, Deserialize)]
struct VndbResults<T> {
    results: Vec<T>,
}
#[derive(Debug, Clone, Deserialize)]
struct VndbCharacter {
    name: String,
    vns: Vec<VndbCharacterRole>,
}
#[derive(Debug, Clone, Deserialize)]
struct VndbCharacterRole {
    id: String,
    ///"main" protagonist, "primary" main character, "side" or "appears"
    role: String,
    spoiler: u8,
}
async fn vndb_query<T: serde::de::DeserializeOwned>(
    endpoint: &str,
    body: serde_json::Value,
) -> Result<Vec<T>, reqwest::Error> {
    let response = reqwest::Client::new()
        .post(format!("https://api.vndb.org/kana/{endpoint}"))
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<VndbResults<T>>()
        .await?;
    Ok(response.results)
}
///fetches length, rating, developers and tags for a vn
pub async fn fetch_vn_details(vndb_id: String) -> Result<Option<VndbDetails>, reqwest::Error> {
    let results = vndb_query::<VndbDetails>(
        "vn",
        serde_json::json!({
            "filters": ["id", "=", vndb_id],
            "fields": "released, length_minutes, length_votes, rating, votecount, developers.name, tags.name, tags.rating, tags.spoiler, tags.category",
        }),
    )
    .await?;
    Ok(results.into_iter().next())
}
///names of the vn's main characters (not the protagonist), which usually have their own routes
pub async fn fetch_route_characters(vndb_id: String) -> Result<Vec<String>, reqwest::Error> {
    let characters = vndb_query::<VndbCharacter>(
        "character",
        serde_json::json!({
            "filters": ["vn", "=", ["id", "=", vndb_id]],
            "fields": "name, vns.id, vns.role, vns.spoiler",
            "results": 100,
        }),
    )
    .await?;
    Ok(characters
        .into_iter()
        .filter(|character| {
            character
                .vns
                .iter()
                .any(|vn| vn.id == vndb_id && vn.role == "primary" && vn.spoiler == 0)
        })
        .map(|character| character.name)
        .collect())
}
