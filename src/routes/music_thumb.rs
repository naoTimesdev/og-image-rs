use std::io::Cursor;

/// Music Thumbnail fetcher for the music player in naoTimes
use axum::{
    extract::{Path, Query},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use image::ImageBuffer;
use lazy_static::lazy_static;
use scraper::Selector;
use serde::Deserialize;
use tracing::info;
use urlencoding::decode;

lazy_static! {
    static ref USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/74.0.3729.115 Safari/537.36";
}

#[derive(Deserialize, Debug)]
pub struct BandcampRequest {
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct SoundcloudRequest {
    artist: String,
    title: String,
}

#[derive(Deserialize, Debug)]
pub struct YTMRequest {
    id: String,
}

fn reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT.to_string())
        .build()
        .unwrap()
}

pub async fn handle_bandcamp_thumb(query: Query<BandcampRequest>) -> Response {
    let decode_url = decode(&query.url)
        .expect("Failed to decode URL")
        .to_string();

    info!("Processing bandcamp URL: {}", decode_url);

    let req = reqwest_client()
        .get(decode_url.clone())
        .send()
        .await
        .unwrap();

    if !req.status().is_success() {
        if req.status() == reqwest::StatusCode::NOT_FOUND {
            return (
                StatusCode::NOT_FOUND,
                format!("Bandcamp not found: `{}`", decode_url),
            )
                .into_response();
        }
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch URL").into_response();
    }

    let html = req.text().await.unwrap();
    let parsed_html = scraper::Html::parse_document(&html);
    let selector = Selector::parse(r#"link[rel="image_src"]"#).unwrap();
    let meta_head = parsed_html.select(&selector).next();

    match meta_head {
        Some(meta) => {
            let href = meta.attr("href").unwrap();
            Redirect::to(href).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Failed to find image").into_response(),
    }
}

pub async fn handle_soundcloud_thumb(request: Path<SoundcloudRequest>) -> Response {
    info!("Processing soundcloud URL: {:?}", request);

    let req = reqwest_client()
        .get(format!(
            "https://soundcloud.com/{}/{}",
            request.artist, request.title
        ))
        .send()
        .await
        .unwrap();

    if !req.status().is_success() {
        if req.status() == reqwest::StatusCode::NOT_FOUND {
            return (
                StatusCode::NOT_FOUND,
                format!(
                    "Soundcloud track not found: `/{}/{}`",
                    request.artist, request.title
                ),
            )
                .into_response();
        }
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch URL").into_response();
    }

    let html = req.text().await.unwrap();

    let parsed_html = scraper::Html::parse_document(&html);
    let selector = Selector::parse(r#"meta[property="og:image"]"#).unwrap();

    let meta_head = parsed_html.select(&selector).next();

    match meta_head {
        Some(meta) => {
            let href = meta.attr("content").unwrap();
            Redirect::to(href).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Failed to find image").into_response(),
    }
}

fn create_ytm_thumb_square(bytes_data: Vec<u8>) -> ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    info!(
        "Creating YouTube Music Thumbnail Square: {} bytes",
        bytes_data.len()
    );

    // Our original image is 1280x720, let's load it into an image buffer
    let mut image = image::load_from_memory(&bytes_data).unwrap();
    // Crop the image to 720x720, with gravity center
    let cropped_image = image::imageops::crop(&mut image, 280, 0, 720, 720).to_image();

    // Save the cropped image into a buffer
    // let mut buf = Cursor::new(Vec::new());

    cropped_image

    // cropped_image
    //     .write_to(&mut buf, image::ImageOutputFormat::Png)
    //     .expect("Unable to write cropped image to buffer");

    // buf.set_position(0);
    // buf.get_ref().to_vec()
}

pub async fn handle_youtube_music_thumb(request: Path<YTMRequest>) -> impl IntoResponse {
    info!("Processing YouTube Music URL: {:?}", request);

    let req = reqwest_client()
        .get(format!(
            "https://i.ytimg.com/vi/{}/maxresdefault.jpg",
            request.id
        ))
        .send()
        .await
        .unwrap();

    let mut headers = HeaderMap::new();

    if !req.status().is_success() {
        if req.status() == reqwest::StatusCode::NOT_FOUND {
            return (
                StatusCode::NOT_FOUND,
                headers,
                format!("YouTube Music track not found: `{}'", request.id)
                    .as_bytes()
                    .to_vec(),
            );
        }
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            headers,
            "Failed to fetch URL".as_bytes().to_vec(),
        );
    }

    let image_data = req.bytes().await.unwrap().to_vec();
    let cropped_image = create_ytm_thumb_square(image_data);

    let mut buf = Cursor::new(Vec::new());
    match cropped_image.write_to(&mut buf, image::ImageOutputFormat::Png) {
        Ok(_) => {
            buf.set_position(0);
            let data = buf.get_ref().to_vec();

            headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
            headers.insert(
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}.thumb.png\"", request.id)
                    .parse()
                    .unwrap(),
            );
            (StatusCode::OK, headers, data)
        }
        Err(err) => {
            tracing::error!("Error writing cropped image to buffer: {}", err);
            headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                "Error writing cropped image to buffer".as_bytes().to_vec(),
            )
        }
    }
}
