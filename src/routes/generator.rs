use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use headless_chrome::{
    protocol::cdp::Page::{CaptureScreenshotFormatOption, Viewport},
    Browser, LaunchOptions,
};
use tokio::task;
use tracing::{error, info};

use crate::generator::utils::get_generator_host;

use super::template::UserCardRequest;

fn generate_screenshot(query_string: String) -> Result<Vec<u8>, anyhow::Error> {
    let options = LaunchOptions::default_builder()
        .build()
        .expect("Couldn't find appropriate Chrome binary.");

    let browser = Browser::new(options).expect("Failed to launch browser.");
    let tab = browser.new_tab().expect("Failed to wait for initial tab.");

    let host_name = get_generator_host();

    let page_to_navigate = format!("{}/_/template/user_card?{}", host_name, query_string);

    let webpage = tab
        .navigate_to(&page_to_navigate)
        .expect("Failed to navigate to page.")
        .wait_until_navigated()
        .expect("Failed to wait until navigated.");

    webpage.wait_for_element("div#ready-notifier").unwrap();
    let body = webpage.find_element("body").unwrap();
    // get body clientWidth and clientHeight

    let mut viewport = Viewport {
        x: 0.0,
        y: 0.0,
        width: 510.0,
        height: 360.0, // temporary
        scale: 1.0,
    };

    let result = body
        .call_js_fn(
            "function() {return [document.body.clientWidth, document.body.clientHeight]}",
            vec![],
            false,
        )
        .unwrap()
        .preview
        .unwrap()
        .properties;

    match &result[1].value {
        Some(val) => match val.parse::<f64>() {
            Ok(parsed) => {
                viewport.height = parsed + 40.0;
            }
            Err(err) => {
                error!("Error parsing height: {}", err);
            }
        },
        None => {
            error!("Error parsing height!");
        }
    }

    info!("Generating screenshot with viewport: {:?}", viewport);

    tab.capture_screenshot(
        CaptureScreenshotFormatOption::Png,
        None,
        Some(viewport),
        true,
    )
}

pub async fn handle_generator_user_card(request: Query<UserCardRequest>) -> impl IntoResponse {
    let res = task::spawn_blocking(move || {
        info!("Generating User Card for: {:?}", request);

        let query_string = serde_qs::to_string(&request.0).unwrap();
        let data = generate_screenshot(query_string);

        match data {
            Ok(data) => {
                info!("Screenshot generated!");
                data
            }
            Err(err) => {
                tracing::error!("Error generating user card: {}", err);
                Vec::new()
            }
        }
    })
    .await;

    let errors_text = "Error generating user card".as_bytes().to_vec();
    let mut headers = HeaderMap::new();

    match res {
        Ok(data) => {
            if data.is_empty() {
                headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
                (StatusCode::INTERNAL_SERVER_ERROR, headers, errors_text)
            } else {
                headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
                let uuid = uuid::Uuid::new_v4().to_string();
                headers.insert(
                    header::CONTENT_DISPOSITION,
                    format!("inline; filename=\"{}.UserCard.png\"", uuid)
                        .parse()
                        .unwrap(),
                );
                // Add cache-control for 10 minutes
                headers.insert(
                    header::CACHE_CONTROL,
                    "public, max-age=600".parse().unwrap(),
                );
                (StatusCode::OK, headers, data)
            }
        }
        Err(err) => {
            tracing::error!("Error generating user card: {}", err);
            headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
            (StatusCode::INTERNAL_SERVER_ERROR, headers, errors_text)
        }
    }
}
