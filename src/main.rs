#[macro_use] extern crate rocket;
use rocket::form::Form;
use rocket_dyn_templates::Template;

#[derive(FromForm)]
struct InputForm {
    username: String,
}

#[get("/")]
fn index() -> Template {
    Template::render("index", &())
}



#[post("/submit", data = "<form_data>")]
fn submit(form_data: Form<InputForm>) -> String {
    format!("You entered: {}", form_data.username)
}


#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, submit])
        .attach(Template::fairing())
}