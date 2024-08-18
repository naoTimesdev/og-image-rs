/// OG Image Generator for naoTimes
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use lazy_static::lazy_static;
use og_image_writer::{style, writer::OGImageWriter, TextArea};
use serde::{Deserialize, Serialize};
use tokio::task;
use tracing::info;

use crate::{report_plausible_event, AppState, PlausibleEvent};

lazy_static! {
    static ref IMAGE_BASE: Vec<u8> = include_bytes!("../../assets/ntui_base.png").to_vec();
    static ref ROBOTO_BOLD: Vec<u8> =
        Vec::from(include_bytes!("../../assets/Roboto-Bold.ttf") as &[u8]);
    static ref ROBOTO_LIGHT: Vec<u8> =
        Vec::from(include_bytes!("../../assets/Roboto-Light.ttf") as &[u8]);
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OGImageRequest {
    name: String,
    count: Option<usize>,
    total: Option<usize>,
}

fn create_og_image(
    uuid: String,
    name: String,
    count: Option<usize>,
    total: Option<usize>,
) -> anyhow::Result<Vec<u8>> {
    let mut writer = OGImageWriter::from_data(
        style::WindowStyle {
            align_items: style::AlignItems::Center,
            justify_content: style::JustifyContent::Center,
            width: 1280,
            height: 720,
            flex_direction: style::FlexDirection::Column,
            ..style::WindowStyle::default()
        },
        &IMAGE_BASE,
        og_image_writer::img::ImageInputFormat::Png,
    )?;

    let mut margin_t = 100;
    if count.is_some() && total.is_some() {
        margin_t += 70;
    } else if count.is_some() || total.is_some() {
        margin_t += 44;
    }

    writer.set_text(
        name.as_str(),
        style::Style {
            font_size: 40.,
            color: style::Rgba([255, 255, 255, 255]),
            text_align: style::TextAlign::Center,
            word_break: style::WordBreak::BreakAll,
            margin: style::Margin(margin_t, 100, 0, 100),
            max_width: Some(1160),
            ..style::Style::default()
        },
        Some(ROBOTO_BOLD.clone()),
    )?;

    if let Some(count) = count {
        let text_data = match count {
            0 => "\nTidak ada utang".to_string(),
            _ => "\nSisa utang: ".to_string(),
        };
        let mut textarea = TextArea::new();
        textarea.push_text(&text_data);
        if count > 0 {
            textarea.push(
                format!("{} utang", count).as_str(),
                style::Style {
                    font_size: 24.,
                    color: style::Rgba([255, 255, 255, 255]),
                    ..style::Style::default()
                },
                Some(ROBOTO_BOLD.clone()),
            )?;
        }
        writer.set_textarea(
            textarea,
            style::Style {
                margin: style::Margin(30, 100, 0, 100),
                font_size: 24.,
                color: style::Rgba([255, 255, 255, 255]),
                text_align: style::TextAlign::Center,
                word_break: style::WordBreak::BreakAll,
                max_width: Some(1160),
                line_height: 2.5,
                ..style::Style::default()
            },
            Some(ROBOTO_LIGHT.clone()),
        )?;
    }

    if let Some(total) = total {
        let text_data = match total {
            0 => "\nTidak ada garapan".to_string(),
            _ => "\nProyek: ".to_string(),
        };
        let mut textarea = TextArea::new();
        textarea.push_text(&text_data);
        if total > 0 {
            textarea.push(
                format!("{} garapan", total).as_str(),
                style::Style {
                    font_size: 24.,
                    color: style::Rgba([255, 255, 255, 255]),
                    ..style::Style::default()
                },
                Some(ROBOTO_BOLD.clone()),
            )?;
        }
        let margin_t2 = if count.is_some() { 12 } else { 30 };
        writer.set_textarea(
            textarea,
            style::Style {
                margin: style::Margin(margin_t2, 100, 0, 100),
                font_size: 24.,
                color: style::Rgba([255, 255, 255, 255]),
                text_align: style::TextAlign::Center,
                word_break: style::WordBreak::BreakAll,
                max_width: Some(1160),
                line_height: 2.5,
                ..style::Style::default()
            },
            Some(ROBOTO_LIGHT.clone()),
        )?;
    }

    // Footer
    let mut footer = TextArea::new();
    footer.push_text("Diprakasai dengan ");
    footer.push(
        "naoTimes",
        style::Style {
            color: style::Rgba([255, 255, 255, 255]),
            font_size: 20.,
            ..style::Style::default()
        },
        Some(ROBOTO_BOLD.clone()),
    )?;
    writer.set_textarea(
        footer,
        style::Style {
            margin: style::Margin(30, 30, 30, 30),
            font_size: 20.,
            color: style::Rgba([255, 255, 255, 128]),
            text_align: style::TextAlign::End,
            right: Some(0),
            bottom: Some(0),
            position: style::Position::Absolute,
            word_break: style::WordBreak::Normal,
            ..style::Style::default()
        },
        Some(ROBOTO_LIGHT.clone()),
    )?;

    info!("Painting: {}", uuid);
    writer.paint()?;
    Ok(writer.encode(og_image_writer::ImageOutputFormat::Png)?)
}

pub async fn handle_og_image_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    og_request: Query<OGImageRequest>,
) -> impl IntoResponse {
    let name = og_request.name.clone();
    let count = og_request.count;
    let total = og_request.total;

    let formatted = serde_qs::to_string(&og_request.0).unwrap_or_default();

    let uuid = uuid::Uuid::new_v4().to_string();

    let res = task::spawn_blocking(move || {
        info!(
            "Generating OG Image for {} with data: {:?}",
            uuid.clone(),
            og_request
        );
        let data = create_og_image(uuid.clone(), name, count, total);

        match data {
            Ok(data) => (data, uuid.clone()),
            Err(err) => {
                tracing::error!("Error creating OG Image: {}", err);
                (Vec::new(), uuid.clone())
            }
        }
    })
    .await;

    let errors_text = "Error creating OG Image".as_bytes().to_vec();

    // format og_request to query string

    // get user agent

    let mut resp_headers = HeaderMap::new();

    match res {
        Ok((data, uuid)) => {
            let ev_metadata: crate::PlausibleMetadata = headers.into();
            let event = PlausibleEvent::default()
                .with_url(format!("/large?{}", formatted))
                .with_props(serde_json::json!({
                    "uuid": uuid.clone(),
                }));
            report_plausible_event(state, event, ev_metadata).await;

            if data.is_empty() {
                resp_headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
                (StatusCode::INTERNAL_SERVER_ERROR, resp_headers, errors_text)
            } else {
                resp_headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
                resp_headers.insert(
                    header::CONTENT_DISPOSITION,
                    format!("inline; filename=\"{}.OGImage.png\"", uuid)
                        .parse()
                        .unwrap(),
                );
                // Add cache-control for 10 minutes
                resp_headers.insert(
                    header::CACHE_CONTROL,
                    "public, max-age=600".parse().unwrap(),
                );
                (StatusCode::OK, resp_headers, data)
            }
        }
        Err(err) => {
            tracing::error!("Error creating OG Image: {}", err);
            resp_headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
            (StatusCode::INTERNAL_SERVER_ERROR, resp_headers, errors_text)
        }
    }
}
