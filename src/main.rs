#![feature(proc_macro_hygiene, decl_macro)]

#[cfg(test)]
mod tests;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate rocket_okapi;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;

use cargo_toml::Manifest;
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use rocket_okapi::swagger_ui::*;
use rocket_prometheus::PrometheusMetrics;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

type ID = usize;
type GreetingMap = Mutex<HashMap<ID, String>>;

#[derive(Serialize, Deserialize, JsonSchema)]
struct Greeting {
    id: Option<ID>,
    message: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Message {
    message: String,
}

#[derive(Serialize)]
struct EnvVar {
    evar: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct HostInfo {
    hostname: String,
}

#[derive(Serialize, Deserialize)]
struct Version {
    version: String,
}

#[get("/")]
fn root() -> Redirect {
    Redirect::to("/v1/greeting")
}

#[openapi]
#[post("/greeting/new", format = "json", data = "<message>")]
fn new(message: Json<Message>, map: State<GreetingMap>) -> Option<Json<Greeting>> {
    let mut hashmap = map.lock().expect("map lock.");
    let id = hashmap.len() + 1;
    let msg = message.0.message;
    hashmap.insert(id, msg.clone());
    Some(Json(Greeting {
        id: Some(id),
        message: msg,
    }))
}

#[openapi]
#[put("/greeting/<id>", format = "json", data = "<message>")]
fn update(id: ID, message: Json<Message>, map: State<GreetingMap>) -> Option<Json<Greeting>> {
    let mut hashmap = map.lock().unwrap();
    let msg = message.0.message;
    if hashmap.contains_key(&id) {
        hashmap.insert(id, msg.clone());
        Some(Json(Greeting {
            id: Some(id),
            message: msg.clone(),
        }))
    } else {
        None
    }
}

#[openapi]
#[get("/greeting")]
fn list(map: State<GreetingMap>) -> Json<Vec<Greeting>> {
    let mut vec = Vec::new();
    let hashmap = map.lock().unwrap();
    for (k, v) in hashmap.iter() {
        vec.push(Greeting {
            id: Some(*k),
            message: v.clone(),
        })
    }
    Json(vec)
}

#[openapi]
#[get("/greeting/<id>", format = "json")]
fn get(id: ID, map: State<GreetingMap>) -> Option<Json<Greeting>> {
    let hashmap = map.lock().unwrap();
    hashmap.get(&id).map(|message| {
        Json(Greeting {
            id: Some(id),
            message: message.clone(),
        })
    })
}

#[openapi]
#[get("/envinfo")]
fn envinfo(map: State<EnvVar>) -> JsonValue {
    json!(map.evar)
}

#[openapi]
#[get("/hostinfo")]
fn hostinfo(host: State<HostInfo>) -> JsonValue {
    json!({
        "hostname": host.hostname
    })
}

#[get("/version")]
fn version(ver: State<Version>) -> JsonValue {
    json!({
        "version": ver.version
    })
}

#[get("/healthz")]
fn health() -> JsonValue {
    json!({
        "status": "200",
        "message": "OK"
    })
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "404",
        "reason": "Resource was not found."
    })
}

fn get_hostinfo() -> HostInfo {
    let contents = fs::read_to_string(String::from("/etc/hostname"))
        .expect("Something went wrong reading the file");
    let chars_to_trim: &[char] = &[' ', '\n'];
    HostInfo {
        hostname: contents.trim_matches(chars_to_trim).to_string(),
    }
}

fn get_envinfo() -> EnvVar {
    let mut map = EnvVar {
        evar: HashMap::new(),
    };
    for (k, v) in env::vars_os() {
        map.evar
            .insert(k.into_string().unwrap(), v.into_string().unwrap());
    }
    map
}

fn get_version() -> Version {
    let manifest = Manifest::from_path(Path::new("Cargo.toml")).unwrap();
    Version {
        version: manifest.package.unwrap().version,
    }
}

fn rocket() -> rocket::Rocket {
    let envmap = get_envinfo();
    let hostname = get_hostinfo();
    let version = get_version();
    let prometheus = PrometheusMetrics::new();
    rocket::ignite()
        .attach(prometheus.clone())
        .mount("/", routes![root, version, health])
        .mount("/metrics", prometheus)
        .mount(
            "/v1",
            routes_with_openapi![new, update, get, list, envinfo, hostinfo],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: Some("/v1/openapi.json".to_owned()),
                urls: None,
            }),
        )
        .register(catchers![not_found])
        .manage(Mutex::new(HashMap::<ID, String>::new()))
        .manage(envmap)
        .manage(hostname)
        .manage(version)
}

fn main() {
    rocket().launch();
}
