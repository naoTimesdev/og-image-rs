use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::prelude::*;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use urlencoding::decode;

use crate::generator::random_status::select_random_status;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);

        tera
    };
}

#[derive(Debug)]
struct UnixTimestamp(u64);
#[derive(Debug)]
struct DiscordFlags(Vec<String>);

#[derive(Serialize)]
struct UserCardTemplate {
    username: String,
    tag: Option<String>,
    nickname: Option<String>,
    role_name: Option<String>,
    created_at: String,
    joined_at: String,
    json_extra: String,
}

#[derive(Serialize)]
struct UserCardJsonExtra {
    flags: Vec<String>,
    role_color: Option<String>,
    status: Option<String>,
    status_text: Option<String>,
    img_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserCardRequest {
    username: String,
    tag: Option<String>,
    nickname: Option<String>,
    role_name: Option<String>,
    role_color: Option<String>,
    created_at: UnixTimestamp,
    joined_at: UnixTimestamp,
    flags: Option<DiscordFlags>,
    status: Option<String>,
    img_url: Option<String>,
}

impl Serialize for UnixTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Serialize for DiscordFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // join with comma
        let joined = self.0.join(",");
        serializer.serialize_str(&joined)
    }
}

// impl trait Deseralize for UnixTimestamp
impl<'de> Deserialize<'de> for UnixTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<UnixTimestamp, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // just convert to u64
        let i = s.parse::<u64>().unwrap();
        Ok(UnixTimestamp(i))
    }
}

impl<'deflag> Deserialize<'deflag> for DiscordFlags {
    fn deserialize<D>(deserializer: D) -> Result<DiscordFlags, D::Error>
    where
        D: serde::Deserializer<'deflag>,
    {
        let s = String::deserialize(deserializer)?;
        // split at , (comma)
        let managed = s.split(",").map(|s| s.to_string()).collect();

        Ok(DiscordFlags(managed))
    }
}

impl UserCardRequest {
    fn to_template(&self) -> UserCardTemplate {
        let mut json_extra = UserCardJsonExtra {
            flags: vec![],
            role_color: self.role_color.clone(),
            status: None,
            status_text: None,
            img_url: None,
        };

        if let Some(flags) = &self.flags {
            json_extra.flags = flags.0.clone();
        }

        if let Some(img_url) = &self.img_url {
            json_extra.img_url = Some(decode(img_url).unwrap().to_string())
        }

        if let Some(status) = &self.status {
            match status.clone().to_ascii_lowercase().as_str() {
                "online" => json_extra.status = Some("online".to_string()),
                "idle" => json_extra.status = Some("idle".to_string()),
                "dnd" => json_extra.status = Some("dnd".to_string()),
                "offline" => json_extra.status = Some("off".to_string()),
                _ => json_extra.status = None,
            }
            match select_random_status(status.clone()) {
                Some(text) => json_extra.status_text = Some(text),
                None => json_extra.status_text = Some("Tidak diketahui".to_string()),
            }
        }

        let stringify = serde_json::to_string(&json_extra).unwrap();

        UserCardTemplate {
            username: self.username.clone(),
            tag: self.tag.clone(),
            nickname: self.nickname.clone(),
            role_name: self.role_name.clone(),
            json_extra: stringify,
            created_at: self.created_at.to_string(),
            joined_at: self.joined_at.to_string(),
            // json_extra: serde_json::to_string(&self).unwrap(),
        }
    }
}

pub async fn handle_template_user_card(query: Query<UserCardRequest>) -> impl IntoResponse {
    let parsed = TEMPLATES.render(
        "user_card_template.html",
        &Context::from_serialize(query.to_template()).unwrap(),
    );

    let mut headers = HeaderMap::new();
    match parsed {
        Ok(data) => {
            headers.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());
            (StatusCode::OK, headers, data)
        }
        Err(err) => {
            // explain more of the error
            tracing::error!("Error rendering template: {:?}", err);
            headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                "Error rendering template".to_string(),
            )
        }
    }
}

// implement UnixTimestamp parse to string
impl std::fmt::Display for UnixTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let i64 = self.0 as i64;

        let naive = NaiveDateTime::from_timestamp_opt(i64, 0).unwrap();
        // convert to UTC
        let utc_dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
        // convert to UTC+7
        let local_dt = utc_dt.with_timezone(&FixedOffset::east_opt(7 * 3600).unwrap());

        write!(f, "{}", local_dt.format("%Y-%m-%d %H:%M:%S WIB"))
    }
}
