#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::response::Redirect;
use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::Template;

use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use rocket::State;

use mongodb::bson::doc;

mod db;
use crate::db::MongoRepo;



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



#[get("/")]
fn login_page() -> Template {
    Template::render("login", ())
}

#[post("/login", data = "<form>")]
async fn handle_login(
    form: Form<LoginForm>,
    repo: &State<MongoRepo>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let username = form.username.to_owned();

    cookies.add(Cookie::new("username", username.clone()));

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
    cookies: &CookieJar<'_>,
) -> Template {
    let username = match cookies.get("username") {
        Some(c) => c.value().to_string(),
        None => return Template::render("login", ()), 
    };

    let users = repo.db.collection::<User>("users");

    let user = users
        .find_one(doc! { "username": &username }, None)
        .await
        .unwrap()
        .unwrap_or(User {
            username: username.clone(),
            data: Some("None".to_string()),
        });

    let mut ctx = HashMap::new();
    ctx.insert("username", user.username);
    ctx.insert("data", user.data.unwrap_or("None".to_string()));

    Template::render("dashboard", &ctx)
}


// GET /submit — Show form
#[get("/submit")]
fn submit_page() -> Template {
    Template::render("submit", ())
}


#[post("/submit", data = "<form>")]
async fn handle_submit(
    form: Form<SubmitForm>,
    repo: &State<MongoRepo>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    let username = cookies.get("username").unwrap().value().to_string();

    let users = repo.db.collection::<User>("users");

    users
        .update_one(
            doc! { "username": &username },
            doc! { "$set": { "data": &form.text }},
            None,
        )
        .await
        .unwrap();

    Redirect::to("/dashboard")
}




#[launch]
async fn rocket() -> _ {
    let repo = MongoRepo::init().await;

    rocket::build()
        .manage(repo)
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
