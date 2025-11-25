#[macro_use] extern crate rocket;
use rocket_dyn_templates::Template;
use std::collections::HashMap;

#[get("/")]
fn login_page() -> Template {
    Template::render("login", ())
}

#[get("/dashboard")]
fn dashboard_page() -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("username", "test_user");
    ctx.insert("data", "None submitted yet");
    Template::render("dashboard", &ctx)
}

#[get("/submit")]
fn submit_page() -> Template {
    Template::render("submit", ())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![login_page, dashboard_page, submit_page])
        .attach(Template::fairing())
}