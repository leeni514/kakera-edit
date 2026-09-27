use reqwest::multipart::{Form, Part};
use std::path::PathBuf;
const CATBOX_UPLOAD_URL: &str = "https://catbox.moe/user/api.php";
///image types discord can show
pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];
///uploads an image to catbox.moe so discord can show it, returns the public url
pub async fn upload_image(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let is_image = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
        .is_some_and(|extension| IMAGE_EXTENSIONS.contains(&extension.as_str()));
    if !is_image {
        return Err("Only PNG, JPG, GIF and WEBP images can be used".into());
    }
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "image.png".to_string());
    let image_bytes = std::fs::read(&path)?;
    let form = Form::new()
        .text("reqtype", "fileupload")
        .part("fileToUpload", Part::bytes(image_bytes).file_name(file_name));
    //catbox drops connections that don't send a user agent
    let response_text = reqwest::Client::builder()
        .user_agent(concat!("Kakera/", env!("CARGO_PKG_VERSION")))
        .build()?
        .post(CATBOX_UPLOAD_URL)
        .multipart(form)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let image_url = response_text.trim().to_string();
    if !image_url.starts_with("https://") {
        return Err(format!("Upload failed: {image_url}").into());
    }
    Ok(image_url)
}
