// src/main.rs
#[macro_use]
extern crate rocket;

mod db;
mod sessions;
mod policies;

use std::collections::HashMap;

use rocket::form::Form;
use rocket::response::Redirect;
use rocket::http::CookieJar;
use rocket::State;
use rocket_dyn_templates::Template;
use serde::{Deserialize, Serialize};

use mongodb::bson::doc;
use base64::{engine::general_purpose, Engine as _};

use db::MongoRepo;
use sessions::{SessionManager};
use policies::{reject_sensitive_text, user_may_access, owner_decrypt_if_allowed, require_authenticated};

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
    pub data: Option<String>,
}

#[derive(Serialize)]
struct SubmitContext {
    policy_error: bool,
    encrypt_error: bool,
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
    let username = form.username.trim().to_string();
    if username.is_empty() {
        return Redirect::to("/");
    }

    sessions.create_session(cookies, &username);

    let users = repo.db.collection::<User>("users");
    users
        .update_one(
            doc! { "username": &username },
            doc! { "$setOnInsert": { "username": &username, "data": "" } },
            mongodb::options::UpdateOptions::builder().upsert(true).build(),
        )
        .await
        .expect("failed to upsert user");

    Redirect::to("/dashboard")
}

type DashboardResult = Result<Template, Redirect>;

#[get("/dashboard")]
async fn dashboard_page(
    repo: &State<MongoRepo>,
    sessions: &State<SessionManager>,
    cookies: &CookieJar<'_>,
) -> DashboardResult {
    let session = match sessions.get_session(cookies) {
        Some(s) => s,
        None => return Err(Redirect::to("/")),
    };

    let users = repo.db.collection::<User>("users");
    let user_opt = users
        .find_one(doc! { "username": &session.username }, None)
        .await
        .expect("db error");

    let (username, data_display) = if let Some(user) = user_opt {
        let data_display = match user.data {
            Some(enc_b64) if !enc_b64.is_empty() => {
                match owner_decrypt_if_allowed(&sessions, &session, &enc_b64) {
                    Ok(plain) => plain,
                    Err(msg) => format!("[policy violation: {}]", msg),
                }
            }
            _ => "(no data stored yet)".to_string(),
        };

        (user.username, data_display)
    } else {
        (session.username.clone(), "(no user document found)".to_string())
    };

    let mut ctx = HashMap::new();
    ctx.insert("username".to_string(), username);
    ctx.insert("data".to_string(), data_display);

    Ok(Template::render("dashboard", &ctx))
}

#[get("/submit?<error>")]
fn submit_page(error: Option<String>) -> Template {
    let ctx = SubmitContext {
        policy_error: matches!(error.as_deref(), Some("sensitive")),
        encrypt_error: matches!(error.as_deref(), Some("encrypt")),
    };
    Template::render("submit", &ctx)
}

#[post("/submit", data = "<form>")]
async fn handle_submit(
    form: Form<SubmitForm>,
    repo: &State<MongoRepo>,
    sessions: &State<SessionManager>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let session = match sessions.get_session(cookies) {
        Some(s) => s,
        None => return Redirect::to("/"),
    };
    if !require_authenticated(Some(&session)) {
        return Redirect::to("/");
    }

    if !reject_sensitive_text(&form.text) {
        return Redirect::to("/submit?error=sensitive");
    }


    if !user_may_access(&session, &session.username) {
        return Redirect::to("/submit?error=not_allowed");
    }

    let encrypted_bytes = match sessions.encrypt_for_session(&session, form.text.as_bytes()) {
        Some(b) => b,
        None => return Redirect::to("/submit?error=encrypt"),
    };

    let encrypted_b64 = general_purpose::STANDARD.encode(encrypted_bytes);

    let users = repo.db.collection::<User>("users");
    users
        .update_one(
            doc! { "username": &session.username },
            doc! { "$set": { "data": encrypted_b64 } },
            None,
        )
        .await
        .expect("failed to update user");

    Redirect::to("/dashboard")
}

#[launch]
fn rocket() -> _ {
    let repo = MongoRepo::init();
    let session_manager = SessionManager::new();

    rocket::build()
        .manage(repo)
        .manage(session_manager)
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
