mod session_management;
mod articles;  // Declares the articles module
use crate::articles::get_articles_handler;

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
use std::fs;
use serde::de::Unexpected::Str;

fn get_main(mut state: State) -> Pin<Box<HandlerFuture>> {


    let f = body::to_bytes(Body::take_from(&mut state)).then(|full_body| match full_body {
        Ok(valid_body) => {

            let _ = String::from_utf8(valid_body.to_vec()).unwrap();
            let uri = Uri::borrow_from(&state).to_string();

            // If we receive additional arguments in the URI we can handle them
            let uri_elements = uri.split("&").collect::<Vec<&str>>();
            //println!("Uri got {} arguments: {:?}", uri_elements.len(), uri_elements);



            let body_content = fs::read_to_string("orailix.com/index.html").unwrap();
            let mut res = create_response(&state, StatusCode::OK, TEXT_HTML, body_content);
            res = header_formatting(res, &state);
            future::ok((state, res))

        }
        Err(e) => future::err((state, e.into())),
    });
    f.boxed()
}

fn _get_page(mut state: State) -> Pin<Box<HandlerFuture>> {
    let f = body::to_bytes(Body::take_from(&mut state)).then(|full_body| match full_body {
        Ok(valid_body) => {
            let _ = String::from_utf8(valid_body.to_vec()).unwrap();
            let uri = Uri::borrow_from(&state).to_string();

            // If we receive additional arguments in the URI we can handle them
            let uri_elements = uri.split("&").collect::<Vec<&str>>();

            let file_location = format!{"orailix.com/{}", uri_elements[0]}.split("?").collect::<Vec<&str>>()[0].to_string();

            let response_content = match fs::read_to_string(file_location.clone()) {
                Ok(body) => {
                    (body, StatusCode::OK)
                }
                Err(e) => {
                    let body = fs::read_to_string("orailix.com/error_page.html").expect("Unable to read file");
                    println!("get page link {} error {:?}", file_location, e);
                    (body, StatusCode::NOT_FOUND)
                }
            };
            let mut res = create_response(&state, response_content.1, TEXT_HTML, response_content.0);
            res = header_formatting(res, &state);
            future::ok((state, res))

        }
        Err(e) => future::err((state, e.into())),
    });
    f.boxed()
}



fn to_dir_handler(mut state: State) -> Pin<Box<HandlerFuture>> {
    let f = body::to_bytes(Body::take_from(&mut state)).then(|full_body| match full_body {
        Ok(_valid_body) => {
            let uri = Uri::borrow_from(&state).to_string();

            // If we receive additional arguments in the URI we can handle them
            let uri_elements = uri.split("&").collect::<Vec<&str>>();

            let file_location = format!{"orailix.com{}", uri_elements[0]}.replace("[\"", "").replace("\"]", "");

            let mut file_location = match file_location.contains('?') {
                true => {file_location.split_once("?").unwrap().0.to_string()},
                false => file_location
            };

            // If no extension in the path, it means we target a folder such as /members/, thus we provide the index.html present
            if file_location.ends_with('/') {
                file_location = format!("{file_location}index.html")
            }

            let file_extension = &file_location.split("/").collect::<Vec<&str>>();
            let file_extension = file_extension.last().unwrap().to_owned();


            let parts = file_extension.split('.').collect::<Vec<&str>>();
            let file_extension = parts.last().unwrap().trim();

            let mut file_name = String::new();
            for p in parts.iter().take(parts.len()) {
                file_name = format!("{file_name}.{p}")
            }

            let mime_type = match file_extension {
                "jpg" => { IMAGE_JPEG}
                "jpeg" => { IMAGE_JPEG}
                "png" => {IMAGE_PNG}
                "svg" => {IMAGE_SVG}
                "html" => { TEXT_HTML}
                "css" => { TEXT_CSS}
                "js" => {TEXT_JAVASCRIPT}
                "xml" => {TEXT_XML}
                "pdf"=> {mime::APPLICATION_PDF}
                _ => {
                    //println!("file location is {}, and file extension is {}", file_location, file_extension);
                    TEXT_PLAIN
                }
            };


            let body_content = match File::open(&file_location) {
                Ok(mut body) => {
                    let mut file_content = Vec::new();
                    match body.read_to_end(&mut file_content) {
                        Ok(_) => file_content,
                        _ => {
                            println!("error reading at {}", file_location);
                            format!("error reading").as_bytes().to_vec()
                        }
                    }
                }
                Err(e) => {
                    println!("error: {e:?}: {file_location}");
                    let mut body = File::open("orailix.com/error_page.html").expect("Unable to read file");
                    //println!("error get support {:?} for {}", e, file_location);
                    let mut file_content = Vec::new();
                    body.read_to_end(&mut file_content).expect("Unable to read");
                    file_content
                }
            };
            let mut res = create_response(&state, StatusCode::OK, mime_type, body_content);
            res = header_formatting(res, &state);
            future::ok((state, res))

        }
        Err(e) => future::err((state, e.into())),
    });
    f.boxed()
}



pub fn main() {
    let cmd: clap::ArgMatches = parse_cmd();
    let addr: String = cmd.value_of("ip").unwrap_or_default().to_string();
    let origin: String = cmd.value_of("origin").unwrap_or_default().to_string();
    println!("The origin of the website is: {}", origin);

    let middleware = match cmd.is_present("https") {
        true => {
            // If Https is enabled, create a secure middleware handling LoginData over sessions
            NewSessionMiddleware::default().with_session_type::<Option<LoginData>>()
        }
        false => {
            NewSessionMiddleware::default()
                // Configure the type of data which we want to store in the session.
                // See the custom_data_type example for storing more complex data.
                .with_session_type::<Option<LoginData>>()
                // By default, the cookies used are only sent over secure connections. For our test server,
                // we don't set up an HTTPS certificate, so we allow the cookies to be sent over insecure
                // connections. This should not be done in real applications.
                .insecure()
        }
    };


    let pipelines = new_pipeline_set();

    let (pipelines, default) = pipelines.add(
        new_pipeline()
            .add(middleware)
            .build(),
    );

    // Creating a pipeline to combine two middleware:
    //  1) Origin tracker for customizing the headers
    //  2) Session Management for login data and more (eg accessing the blog utilities)
    let origin_domain = OriginDomain::new(origin);
    let (pipelines, extended) = pipelines.add(
        new_pipeline()
            .add(StateMiddleware::new(origin_domain))
            .build(),
    );

    let pipeline_set = finalize_pipeline_set(pipelines);
    let default_chain = (default, ());
    let extended_chain = (extended, default_chain);


    let router = build_router(extended_chain, pipeline_set, |route| {
        // You can add a `to_dir` or `to_file` route simply using a
        // `String` or `str` as above, or a `Path` or `PathBuf` to accept
        // default options.
        //println!("{:?}", );

        route.scope("/api/articles/", |route| {
            route.get("").to(get_articles_handler);
        });

        route.scope("/", |route| {
            route.get("").to(get_main);
        });
        
        route.get("/*").to(to_dir_handler);


    });

    if cmd.is_present("https") {
        // TLS gotham server that load the .pem files
        gotham::start_with_tls(addr, router, build_config().unwrap())
    } else {
        // Gotham HTTP
        gotham::start(addr, router)
    }
}




// Load the certificates
fn build_config() -> Result<rustls::ServerConfig, rustls::TLSError> {
    let mut cfg = rustls::ServerConfig::new(NoClientAuth::new());
    let full_chain = File::open("/etc/letsencrypt/live/orailix.com/fullchain.pem").unwrap();
    let mut cert_file = BufReader::new(full_chain);
    let priv_key = File::open("/etc/letsencrypt/live/orailix.com/privkey.pem").unwrap();
    let mut key_file = BufReader::new(priv_key);
    let certs = certs(&mut cert_file).unwrap();

    let mut keys = pkcs8_private_keys(&mut key_file).unwrap();

    cfg.set_single_cert(certs, keys.remove(0))?;
    Ok(cfg)
}

pub fn parse_cmd() -> clap::ArgMatches<'static> {
    let matches = App::new("")
        .arg(Arg::with_name("ip")
            .short("ip")
            .long("ip")
            .value_name("String")
            .help("Bind to tihs [ip:port] of your server")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("https")
            .short("https")
            .long("https")
            .help("Run with https enabled"))
        .arg(Arg::with_name("origin")
            .short("origin")
            .long("origin")
            .help("Specifies Access-Control-Allow-Origin")
            .required(true)
            .takes_value(true))
        .get_matches();

    println!("{:?}", matches);

    return matches;
}

