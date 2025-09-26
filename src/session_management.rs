use serde::{Deserialize, Serialize};
use gotham::state::{FromState, State};
use chrono::Utc;
use gotham_derive::StateData;
use gotham::helpers::http::response::create_response;
use gotham::handler::HandlerFuture;
use std::pin::Pin;
use mime::{TEXT_HTML, TEXT_PLAIN};
use gotham::hyper::{body, Body, Response, Uri, StatusCode};
use futures_util::{future, FutureExt};
use gotham::middleware::session::{SessionData};



#[derive(Clone, Deserialize, Serialize)]
pub struct LoginData {
    pub(crate) user_id: String,
    pub(crate) connected: bool,
    last_interaction: String
}

pub fn connect_user(mut state: State) -> Pin<Box<HandlerFuture>> {
    let f = body::to_bytes(Body::take_from(&mut state)).then(|full_body| match full_body {
        Ok(valid_body) => {

            println!("Entered 'connect_user' backend function");

            // 1 - Parse the body's payload to extract the login credentials
            // a - To get the URI, which is basically what was written in the request's URL you can extract it with:
            //      let uri = Uri::borrow_from(&state).to_string();
            //      The way it works is simply by using gotham's URI borrow function on the state, and turn it into a string.
            //      From there you can do operations such as parsing, splitting etc to collect data.
            //
            // b - To get the internal payload, you can use:
            //     let extracted_body = String::from_utf8(valid_body.to_vec()).unwrap();
            //      It will extract the body of the sent request as a string, and again you can do operations on that.
            //      This internal data is created from the user or browser by a XML request. We can do them in Javascript and you can
            //      see an example in the login webpage at line XXX. We basically create a request by specifying a URL (which leads to connect_user)
            //      and we add some payload data that you are trying to extract here.

            // 2 - Checks if the credentials are correct
            //      From the previously extracted body, parse the username and password and match it.
            //      You can use the 'is_credential_valid' function to check if they are correct
            // a - If they are wrong, you should return an error message in a string with a Status Error from the function
            // b - If they are correct, you should proceed to 3)

            // 3 - The credentials are correct, so you should change the internal state of LoginData's 'connected' value from false to true
            //     For that, refer to 'https://github.com/gotham-rs/gotham/blob/main/examples/sessions/custom_data_type/src/main.rs'
            //      especially at the lines 38-48 where the internal's state is extracted. In our case we are not using the 'VisitData' structure,
            //      but the custom one called 'LoginData' that we have implemented ourselves.
            //      You should change the internal state to true after verifying the credentials.


            let response_payload = format!("your payload that you should create");
            let mut res = create_response(&mut state, StatusCode::OK, TEXT_HTML, response_payload);
            future::ok((state, res))
        }
        Err(e) => future::err((state, e.into())),
    });

    f.boxed()
}

// Check if credentials are valid
// We have some default login credentials
fn is_credential_valid(username: &str, password: &str) -> bool {
    return if username == "Spock" && password == "enigma42" { true }
    else { false }
}


pub fn is_user_connected(mut state: State) -> Pin<Box<HandlerFuture>> {
    let f = body::to_bytes(Body::take_from(&mut state)).then(|full_body| match full_body {
        Ok(valid_body) => {

            let login_data: &mut Option<LoginData> = SessionData::<Option<LoginData>>::borrow_mut_from(&mut state);
            let is_user_connected = match login_data {
                Some(ref login_data) => login_data.connected,
                None => false,
            };

            if is_user_connected {
                // User is connected
                let response_payload = format!("user is connected");
                let mut res = create_response(&mut state, StatusCode::OK, TEXT_HTML, response_payload);
                future::ok((state, res))
            } else {
                let response_payload = format!("user is NOT connected");
                let mut res = create_response(&mut state, StatusCode::OK, TEXT_HTML, response_payload);
                future::ok((state, res))
            }

        }
        Err(e) => future::err((state, e.into())),
    });

    f.boxed()
}


// OriginDomain struct that will keep in shared memory the origin domain access for header creation.
// This is the middlewares structure
// Todo: this will be merged with other branches' middlewares structures
#[derive(Clone, StateData)]
pub struct OriginDomain {
    pub origin_domain: String,
}

/// Counter implementation.
impl OriginDomain {
    /// Creates a new origin domain to allow cross domain sharing
    pub(crate) fn new(origin: String) -> Self {
        Self {
            origin_domain: origin,
        }
    }
}

// Get the domain of the website passed as a command line argument during initialization
pub fn get_domain_origin(state: &State) -> String {
    let origin_domain = OriginDomain::borrow_from(&state).origin_domain.clone();
    return origin_domain
}

// Format the header according to our specification
pub fn header_formatting(mut res: Response<Body>, state: &State) -> Response<Body> {

    let utc = Utc::now();

    let headers = res.headers_mut();
    headers.insert("Strict-Transport-Security", "max-age=63072000".parse().unwrap());
    headers.insert("X-Frame-Options", "SAMEORIGIN".parse().unwrap());
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("Access-Control-Allow-Origin", get_domain_origin(&state).parse().unwrap());
    headers.insert("Vary", "Origin".parse().unwrap());
    headers.insert("Permissions-Policy", "accelerometer=(), ambient-light-sensor=(), autoplay=(), battery=(), camera=(), cross-origin-isolated=(), display-capture=(), document-domain=(), encrypted-media=(), execution-while-not-rendered=(), execution-while-out-of-viewport=(), fullscreen=(), geolocation=(), gyroscope=(), keyboard-map=(), magnetometer=(), microphone=(), midi=(), navigation-override=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), screen-wake-lock=(), sync-xhr=(), usb=(), web-share=(), xr-spatial-tracking=()".parse().unwrap());
    headers.insert("Date", format!("{:?}", utc).parse().unwrap());
    return res
}