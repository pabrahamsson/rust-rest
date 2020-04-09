use crate::rocket;
use rocket::http::{ContentType, Status};
use rocket::local::Client;
use serde::Deserialize;
use serde_json::{Result, Value};

type ID = usize;

#[derive(Deserialize)]
struct Greeting {
    id: Option<ID>,
    message: String,
}

#[test]
fn root_redirect() {
    let client = Client::new(rocket()).unwrap();

    // Check that / redirects correctly
    let response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::SeeOther);
    assert_eq!(response.headers().get_one("Location"), Some("/v1/greeting"));
}

#[test]
fn empty_greeting() {
    let client = Client::new(rocket()).unwrap();

    // Check that we get an empty json array.
    let mut response = client
        .get("/v1/greeting")
        .header(ContentType::JSON)
        .dispatch();
    let _body = response.body_string().unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert!(_body.contains("[]"));
}

#[test]
fn bad_get_put() {
    let client = Client::new(rocket()).unwrap();

    // Try to get a greeting that doesn't exist.
    let mut response = client
        .get("/v1/greeting/37")
        .header(ContentType::JSON)
        .dispatch();
    let _body = response.body_string().unwrap();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert!(_body.contains("Resource was not found."));

    // Try to get a greeting with an invalid ID.
    let mut response = client
        .get("/v1/greeting/invalid")
        .header(ContentType::JSON)
        .dispatch();
    let _body = response.body_string().unwrap();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert!(_body.contains("Resource was not found."));

    // Try to put a greeting without a proper body.
    let response = client
        .put("/v1/greeting/33")
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);

    // Try to put a greeting for an ID that doesn't exist.
    let response = client
        .put("/v1/greeting/33")
        .header(ContentType::JSON)
        .body(r#"{ "message": "Cargo testing!!" }"#)
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
}

#[test]
fn post_get_put_get() -> Result<()> {
    let client = Client::new(rocket()).unwrap();

    // Check that a greeting with ID 1 doesn't exist.
    let response = client
        .get("/v1/greeting/1")
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );

    // Add a new greeting
    let mut response = client
        .post("/v1/greeting/new")
        .header(ContentType::JSON)
        .body(r#"{ "message": "Cargo testing!"}"#)
        .dispatch();
    let _body = response.body_string().unwrap();
    let g: Greeting = serde_json::from_str(&_body)?;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert_eq!(g.id, Some(1));
    assert_eq!(g.message, String::from("Cargo testing!"));

    // Check that the greeting exists with the correct content.
    let mut response = client
        .get("/v1/greeting/1")
        .header(ContentType::JSON)
        .dispatch();
    let _body = response.body_string().unwrap();
    let g: Greeting = serde_json::from_str(&_body)?;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert_eq!(g.id, Some(1));
    assert_eq!(g.message, String::from("Cargo testing!"));

    // Change the greeting content
    let mut response = client
        .put("/v1/greeting/1")
        .header(ContentType::JSON)
        .body(r#"{ "message": "Updated greeting" }"#)
        .dispatch();
    let _body = response.body_string().unwrap();
    let g: Greeting = serde_json::from_str(&_body)?;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert_eq!(g.id, Some(1));
    assert_eq!(g.message, String::from("Updated greeting"));

    // Check that the greeting exists with the updated content.
    let mut response = client
        .get("/v1/greeting/1")
        .header(ContentType::JSON)
        .dispatch();
    let _body = response.body_string().unwrap();
    let g: Greeting = serde_json::from_str(&_body)?;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert_eq!(g.id, Some(1));
    assert_eq!(g.message, String::from("Updated greeting"));
    assert!(_body.contains("Updated greeting"));
    Ok(())
}

#[test]
fn get_envinfo() {
    let client = Client::new(rocket()).unwrap();

    // Check that we get a non empty json response
    let mut response = client.get("/v1/envinfo").dispatch();
    let _body = response.body_string().unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert!(!_body.is_empty());
}

#[test]
fn get_hostinfo() {
    let client = Client::new(rocket()).unwrap();

    // Check that we get a non empty json response
    let mut response = client.get("/v1/hostinfo").dispatch();
    let _body = response.body_string().unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert!(!_body.is_empty());
}

#[test]
fn get_openapi_json() -> Result<()> {
    let client = Client::new(rocket()).unwrap();

    // Check that we get a non empty json response
    let mut response = client.get("/v1/openapi.json").dispatch();
    let _body = response.body_string().unwrap();
    let j: Value = serde_json::from_str(&_body)?;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get_one("Content-Type"),
        Some("application/json")
    );
    assert_eq!(j["openapi"].as_str(), Some("3.0.0"));
    Ok(())
}
