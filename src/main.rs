#[macro_use]
extern crate rocket;
extern crate dotenv;

use dotenv::dotenv;
use std::env;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io;

use rocket::Request;
use rocket::http::Status;
use rocket::tokio::task::spawn_blocking;
use rocket::tokio::time::{sleep, Duration};
use rocket::fs::{NamedFile, FileServer, relative};
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket_dyn_templates::Template;

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
struct RawTeapotJson(String);

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Product<'a> {
  name: &'a str,
  sort: usize
}

// #[get("/")]
// fn index() -> &'static str {
//   "hello"
// }

#[get("/info")]
fn info() -> &'static str {
  "build server rocket verion 0.5.0-rc.1"
}

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
  sleep(Duration::from_secs(seconds)).await;
  format!("Waiting {} seconds", seconds)
}

#[get("/blocking_task")]
async fn blocking_task() -> io::Result<Vec<u8>> {
  let vec = spawn_blocking(|| std::fs::read("data.txt")).await
            .map_err(|e| io::Error::new(io::ErrorKind::Interrupted, e))??;
  Ok(vec)
}

#[get("/news/<_>/<id>")]
fn news_detail_int(id: usize) -> String {
  format!("News detail {}", id)
}

#[get("/news/<_>/<id>", rank = 2)] // 
fn news_detail_str(id: &str) -> String {
  format!("News detail {}", id)
}

#[get("/user/<id>")]
fn user(id: usize) -> String {
  format!("User: {}", id)
}

#[get("/user/<id>", rank = 2)]
fn user_int(id: isize) -> String {
  format!("User: {}", id)
}

#[get("/user/<id>", rank = 3)]
fn user_str(id: &str) -> String {
  format!("User: {}", id)
}

// #[get("/")]
// fn docs() -> &'static str {
//   "wellcome to API document"
// }

#[get("/product/json")]
fn product_json() -> Json<Product<'static>> {
  let product = Product {
    name: "banana",
    sort: 1
  };

  Json(product)
}

#[get("/template")]
fn template() -> Template {
  let mut context: HashMap<&str, &str> = HashMap::new();
  context.insert("name", "HTML content from render template");

  Template::render("index", &context)
}

#[get("/json")]
fn json() -> RawTeapotJson {
  RawTeapotJson(format!("{{\"{}\": \"{}\"}}", "name", "rocket"))
}

#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
  let mut path = Path::new(relative!("public")).join(file);
  if path.is_dir() {
    path.push("index.html");
  }

  NamedFile::open(path).await.ok()
}

#[catch(404)]
fn not_found(status: Status, req: &Request) -> String {
  format!("#{}, file not found '{}'", status, req.uri())
}

// #[launch]
// fn rocket() -> _ {
//   rocket::build()
//   .register("/", catchers![not_found])
//   .mount("/", routes![index, info])
//   .mount("/docs", routes![docs])
// }

#[rocket::main]
async fn main() {
  dotenv().ok();

  let _ = rocket::build()
    .register("/", catchers![not_found])
    .mount("/", routes![files, info, delay, blocking_task, news_detail_int, news_detail_str, user, user_int, user_str, json, product_json, template]) // 
    .mount("/", FileServer::from(relative!("public")))
    // .mount("/docs", routes![docs])
    .attach(Template::fairing())
    .launch()
    .await;
}
