use chrono::NaiveDate;
use serde::Serialize;
use std::{fs, path::Path};
use clap::{App, Arg};
use std::fs::File;
use std::io::{BufReader, Read};
use gotham::pipeline::set::{new_pipeline_set, finalize_pipeline_set};
use gotham::pipeline::new_pipeline;
use gotham::middleware::state::StateMiddleware;
use gotham::middleware::session::{NewSessionMiddleware};
use gotham::router::builder::{build_router, DrawRoutes, DefineSingleRoute};
use crate::session_management::{LoginData, OriginDomain, header_formatting};
use gotham::rustls;
use gotham::rustls::NoClientAuth;
use gotham::rustls::internal::pemfile::{certs, pkcs8_private_keys};

use gotham::state::{FromState, State};
use std::pin::Pin;
use gotham::handler::HandlerFuture;
use gotham::hyper::{body, Body, Uri, StatusCode};
use gotham::helpers::http::response::create_response;
use mime::{TEXT_HTML, IMAGE_JPEG, IMAGE_PNG, IMAGE_SVG, TEXT_CSS, TEXT_JAVASCRIPT, TEXT_XML, TEXT_PLAIN};
use futures_util::{future, FutureExt};
use std::collections::HashMap;


#[derive(Serialize)]
struct Article {
    title: String,
    date: String,
    formatted_date: String,
    picture_url: String,
    category: String,
    link: String,
}


// Add at top of file
#[derive(Default)]
struct Manifest {
    title: String,
    date: String,
    picture: String,
    page: String,
}

pub fn get_articles_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let f = async move {
        // Parse query parameters correctly
        let uri = Uri::borrow_from(&state);
        let query = uri.query().unwrap_or_default();
        let mut params = HashMap::new();
        for pair in query.split('&') {
            let mut kv = pair.splitn(2, '=');
            if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                params.insert(k, v);
            }
        }

        let requested_categories: Vec<&str> = params.get("categories")
            .map(|s| s.split(',').collect())
            .unwrap_or_default();

        let news_path = "orailix.com/news";
        let mut articles = Vec::new();

        // Directory traversal logic
        if let Ok(categories) = fs::read_dir(news_path) {
            for category_entry in categories.flatten() {
                let category_path = category_entry.path();
                let category_name = category_path.file_name().unwrap().to_string_lossy();

                if !requested_categories.is_empty() && !requested_categories.contains(&category_name.as_ref()) {
                    continue;
                }

                if let Ok(articles_entries) = fs::read_dir(category_path.to_str().unwrap()) {
                    for article_entry in articles_entries.flatten() {
                        let article_path = article_entry.path();
                        let article_folder = article_path.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned();

                        // In the article creation code:
                        if let Some(manifest) = parse_manifest(&article_path) {
                            // In the article creation block:
                            let (link, picture_url) = if !manifest.page.is_empty() {
                                // External page - direct URLs
                                (
                                    manifest.page.clone(),
                                    manifest.picture.clone()
                                )
                            } else {
                                // Local article - verify path exists
                                let local_image = if manifest.picture.starts_with("http") {
                                    manifest.picture.clone()
                                } else {
                                    // Construct relative path
                                    let rel_path = format!(
                                        "/news/{}/{}/{}",
                                        category_name,
                                        article_folder,
                                        manifest.picture
                                    );

                                    // Verify file exists
                                    let abs_path = format!(
                                        "orailix.com/news/{}/{}/{}",
                                        category_name,
                                        article_folder,
                                        manifest.picture
                                    );

                                    if Path::new(&abs_path).exists() {
                                        rel_path
                                    } else {
                                        // Fallback to placeholder or empty
                                        "/static/image-not-found.png".to_string()
                                    }
                                };

                                (
                                    format!("/news/{}/{}/", category_name, article_folder),
                                    local_image
                                )
                            };

                            let formatted_date = match NaiveDate::parse_from_str(&manifest.date, "%Y-%d-%m") {
                                Ok(date) => date.format("%b %d, %Y").to_string(), // <-- Created here
                                Err(e) => {
                                    println!("Error parsing date {}: {}", manifest.date, e);
                                    continue; // Skip invalid dates
                                }
                            };

                            // Then used in the Article struct
                            articles.push(Article {
                                title: manifest.title,
                                date: manifest.date.clone(),
                                formatted_date, // <-- Used here
                                picture_url,
                                category: category_name.to_string(),
                                link,
                            });
                        }
                    }
                }
            }
        }

        // Add limit parameter handling
        let limit: Option<usize> = params.get("limit")
            .and_then(|l| l.parse().ok());
        // After collecting all articles:
        // Sort articles by date descending
        articles.sort_by(|a, b| {
            let a_date = NaiveDate::parse_from_str(&a.date, "%Y-%d-%m").unwrap();
            let b_date = NaiveDate::parse_from_str(&b.date, "%Y-%d-%m").unwrap();
            b_date.cmp(&a_date)
        });
        // Apply limit if specified
        if let Some(l) = limit {
            articles.truncate(l);
        }

        // Manual JSON serialization with proper escaping
        let mut json = String::from("[");
        for (i, article) in articles.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(
                r#"{{"title":"{}","date":"{}","formatted_date":"{}","picture_url":"{}","category":"{}","link":"{}"}}"#,
                article.title.replace('"', "\\\""),
                article.date.replace('"', "\\\""),
                article.formatted_date.replace('"', "\\\""),
                article.picture_url.replace('"', "\\\""),
                article.category.replace('"', "\\\""),
                article.link.replace('"', "\\\"")
            ));
        }
        json.push(']');

        let mut res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, json);
        res = header_formatting(res, &state);
        Ok((state, res))
    };
    f.boxed()
}

fn parse_manifest(path: &Path) -> Option<Manifest> {
    let manifest_path = path.join("manifest.txt");
    let content = fs::read_to_string(manifest_path).ok()?;

    let mut manifest = Manifest::default();
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            match parts[0].trim() {
                "title" => manifest.title = parts[1].trim().to_string(),
                "date" => manifest.date = parts[1].trim().to_string(),
                "picture" => manifest.picture = parts[1].trim().to_string(),
                "page" => manifest.page = parts[1].trim().to_string(),
                _ => {}
            }
        }
    }

    if !manifest.title.is_empty() {
        Some(manifest)
    } else {
        None
    }
}