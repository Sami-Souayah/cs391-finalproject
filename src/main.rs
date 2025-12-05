#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::response::Redirect;
use rocket::http::CookieJar;
use rocket_dyn_templates::Template;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rocket::State;

use mongodb::bson::doc;

mod db;
mod sessions;
mod policies;

use db::MongoRepo;
use sessions::{SessionManager};
use policies::{reject_sensitive_text, user_may_access};

#[derive(FromForm)]
struct LoginForm {
    username: String,
}

#[derive(FromForm)]
struct SubmitForm {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub data: Option<String>,  // encrypted string
}

#[get("/")]
fn login_page() -> Template {
    Template::render("login", ())
}

#[post("/login", data = "<form>")]
async fn handle_login(
    form: Form<LoginForm>,
    repo: &State<MongoRepo>,
    sessions: &State<SessionManager>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let username = form.username.to_owned();

    // Create secure session
    sessions.create_session(cookies, &username);

    let users = repo.db.collection::<User>("users");

    users
        .update_one(
            doc! { "username": &username },
            doc! { "$setOnInsert": { "data": "" }},
            mongodb::options::UpdateOptions::builder()
                .upsert(true)
                .build(),
        )
        .await
        .unwrap();

    Redirect::to("/dashboard")
}

#[get("/dashboard")]
async fn dashboard_page(
    repo: &State<MongoRepo>,
    sessions: &State<SessionManager>,
    cookies: &CookieJar<'_>,
) -> Template {
    let session = match sessions.get_session(cookies) {
        Some(s) => s,
        None => return Template::render("login", ()),
    };

    let users = repo.db.collection::<User>("users");

    let user = users
        .find_one(doc! { "username": &session.username }, None)
        .await
        .unwrap()
        .unwrap_or(User {
            username: session.username.clone(),
            data: Some("None".into()),
        });

    let mut ctx = HashMap::new();
    ctx.insert("username", user.username);
    ctx.insert("data", user.data.unwrap_or("None".to_string()));

    Template::render("dashboard", &ctx)
}

#[get("/submit")]
fn submit_page() -> Template {
    Template::render("submit", ())
}

#[post("/submit", data = "<form>")]
async fn handle_submit(
    form: Form<SubmitForm>,
    repo: &State<MongoRepo>,
    sessions: &State<SessionManager>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let session = sessions.get_session(cookies).unwrap();

    // Sensitive data validation
    if !reject_sensitive_text(&form.text) {
        panic!("Sensitive data detected! Rejecting.");
    }

    // Encrypt data before storing
    let encrypted = base64::encode(&sessions.cocoon.wrap(form.text.as_bytes()).unwrap());

    let users = repo.db.collection::<User>("users");

    users
        .update_one(
            doc! { "username": &session.username },
            doc! { "$set": { "data": encrypted }},
            None,
        )
        .await
        .unwrap();

    Redirect::to("/dashboard")
}

#[launch]
async fn rocket() -> _ {
    let repo = MongoRepo::init().await;
    let session_mgr = SessionManager::new();

    rocket::build()
        .manage(repo)
        .manage(session_mgr)
        .mount(
            "/",
            routes![
                login_page,
                handle_login,
                dashboard_page,
                submit_page,
                handle_submit
            ],
        )
        .attach(Template::fairing())
}