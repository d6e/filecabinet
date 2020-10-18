#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use error_chain::error_chain;
use glob::glob;
use itertools::chain;
use rocket::request::Form;
use rocket::response::content;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::templates::Template;
use std::env;
use std::error::Error;
#[allow(dead_code)]
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use rocket_contrib::serve::StaticFiles;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::Path;
mod crypto;
mod cli;

#[derive(FromForm, Clone)]
struct Document {
    orig_name: String,
    date: String,
    institution: String,
    name: String,
    page: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = cli::get_program_input();
    if config.launch_web {
        rocket::ignite()
            .mount("/node_modules", StaticFiles::from("node_modules"))
            .mount("/static", StaticFiles::from("static"))
            .mount("/documents", StaticFiles::from("documents"))
            .mount("/", routes![index, get_docs, get_doc, new])
            .manage(config)
            .attach(Template::fairing())
            .launch();
    }
    Ok(())
}

#[derive(Serialize)]
struct Context {
  filename: String,
  date: String,
  files: Vec<String>
}

#[get("/")]
fn index(config: State<cli::Config>) -> Template {
    let now: DateTime<Utc> = Utc::now();
    let files: Vec<String> = list_files(&PathBuf::from(&config.target_directory));
    let context = Context {
        filename: "uboot.pdf".to_string(),
        date: now.format("%Y-%m-%d").to_string(),
        files: files,
    };
    Template::render("index", &context)
}

#[get("/doc")]
fn get_docs(config: State<cli::Config>) -> JsonValue {
    println!("GET /doc -- listing files in '{}'", &config.target_directory);
    let docs = list_files(&PathBuf::from(&config.target_directory));
    println!("docs={:?}", docs);
    JsonValue(serde_json::json!(docs))
}

#[get("/doc/<name>")]
fn get_doc(name: String) -> String {
    format!("Hello, {}!", name.as_str())
}

#[post("/doc", data = "<doc>")]
fn new(doc: Form<Document>) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(format!(
        "static/{}_{}_{}_{}.cocoon",
        doc.date, doc.institution, doc.name, doc.page
    ))?;

    let mut f = File::open(format!("static/{}",&doc.orig_name))?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    crypto::encrypt_file(&mut file, buffer)?;
    Ok(())
}

fn list_files(path: &PathBuf) -> Vec<String> {
    if !path.exists() {
        return Vec::new(); // TODO: turn this into an optional
    }
    path.read_dir()
        .expect("read_dir call failed")
        .map(|x| x.unwrap().path())
        .filter(|x| x.extension().unwrap() == "pdf" || x.extension().unwrap() == "jpg" || x.extension().unwrap() == "png")
        .map(|x| x.file_name().unwrap().to_str().unwrap().to_owned())
        .collect()
}
