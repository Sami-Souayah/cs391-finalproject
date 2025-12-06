// main.rs
#[macro_use]
extern crate rocket;

mod db;
mod sessions;
mod policies;

use rocket::form::Form;
use rocket::response::Redirect;
use rocket::http::CookieJar;
use rocket_dyn_templates::Template;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rocket::State;
use base64::Engine as _;

use mongodb::bson::doc;
use mongodb::options::UpdateOptions;



use db::MongoRepo;
use sessions::{SessionManager, SessionData};
use policies::{reject_sensitive_text, owner_decrypt_if_allowed};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    // base64-encoded ciphertext of the user's data
    data: Option<String>,
}

#[derive(FromForm)]
struct LoginForm {
    username: String,
}

#[derive(FromForm)]
struct SubmitForm {
    text: String,
}

// ---------- Routes ----------

#[get("/login")]
fn login_page() -> Template {
    Template::render("login", &HashMap::<String, String>::new())
}

#[post("/login", data = "<form>")]
async fn handle_login(
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
    sessions: &State<SessionManager>,
    repo: &State<MongoRepo>,
) -> Redirect {
    let username = form.username.trim().to_string();
    if username.is_empty() {
        return Redirect::to("/login");
    }

    let session = SessionData { username: username.clone() };
    sessions.set_session_cookie(cookies, &session);

    let users = repo.db.collection::<User>("users");
    let filter = doc! { "username": &username };
    let update = doc! { "$setOnInsert": { "username": &username } };
    let options = mongodb::options::UpdateOptions::builder().upsert(true).build();

    // OLD: users.update_one(...).await;
    let _ = users.update_one(filter, update, options);

    Redirect::to("/dashboard")
}

#[get("/dashboard")]
async fn dashboard_page(
    cookies: &CookieJar<'_>,
    sessions: &State<SessionManager>,
    repo: &State<MongoRepo>,
) -> Result<Template, Redirect> {
    let session = match sessions.get_session_from_cookies(cookies) {
        Some(s) => s,
        None => return Err(Redirect::to("/login")),
    };

    let users = repo.db.collection::<User>("users");

    let user = users
        .find_one(doc! { "username": &session.username }, None)
        .ok()
        .flatten();
    let data_display = if let Some(user) = user {
        if let Some(enc) = user.data {
            match owner_decrypt_if_allowed(&sessions, &session, &enc) {
                Ok(plain) => plain,
                Err(_) => "[policy violation: cannot decrypt]".to_string(),
            }
        } else {
            "(no data submitted yet)".to_string()
        }
    } else {
        "(no user document found)".to_string()
    };

    let mut ctx = HashMap::new();
    ctx.insert("username".to_string(), session.username.clone());
    ctx.insert("data".to_string(), data_display);

    Ok(Template::render("dashboard", &ctx))
}

#[get("/submit")]
fn submit_page() -> Template {
    Template::render("submit", &HashMap::<String, String>::new())
}

#[post("/submit", data = "<form>")]
async fn handle_submit(
    form: Form<SubmitForm>,
    cookies: &CookieJar<'_>,
    sessions: &State<SessionManager>,
    repo: &State<MongoRepo>,
) -> Redirect {
    let session = match sessions.get_session_from_cookies(cookies) {
        Some(s) => s,
        None => return Redirect::to("/login"),
    };

    let text = form.text.trim().to_string();
    if text.is_empty() {
        return Redirect::to("/submit");
    }

    if !reject_sensitive_text(&text) {
        return Redirect::to("/submit?error=policy");
    }

    let encrypted_bytes = match sessions.encrypt_for_session(&session, text.as_bytes()) {
        Some(b) => b,
        None => return Redirect::to("/submit?error=encrypt"),
    };

    let encrypted_b64 =
        base64::engine::general_purpose::STANDARD.encode(encrypted_bytes);

    let users = repo.db.collection::<User>("users");
    let _ = users.update_one(
        doc! { "username": &session.username },
        doc! { "$set": { "data": encrypted_b64 } },
        None,
    );

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
