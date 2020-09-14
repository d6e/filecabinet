#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use clap::{value_t, App, Arg, SubCommand};
use cocoon::{Cocoon, Creation};
use error_chain::error_chain;
use glob::glob;
use itertools::chain;
use rand::rngs::ThreadRng;
use rocket::request::Form;
use rocket::response::content;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::templates::Template;
use serde;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs;
#[allow(dead_code)]
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use rocket_contrib::serve::StaticFiles;
use chrono::{DateTime, Utc};


#[derive(FromForm, Clone)]
struct Document {
    orig_name: String,
    date: String,
    institution: String,
    name: String,
    page: String,
}

struct Config {
    verbose: bool,
    launch_web: bool,
    target_directory: String,
}

fn get_program_input() -> Config {
    let name_verbose = "verbose";
    let name_launch_web = "web";
    let name_target_directory = "target-directory";
    let default_target_directory = String::new();
    let matches = App::new("filecabinet")
        .version("1.0")
        .author("Danielle <filecabinet@d6e.io>")
        .about("Filecabinet - A relatively secure solution to managing scanned files.")
        .arg(
            Arg::with_name(name_verbose)
                .short("v")
                .long(name_verbose)
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name(name_launch_web)
                .short("w")
                .long(name_launch_web)
                .help("Launches the web server."),
        )
        .arg(
            Arg::with_name(name_target_directory)
                .short("d")
                .long(name_target_directory)
                .takes_value(true)
                .value_name("DIR")
                .help("Target directory for archival."),
        )
        .get_matches();
    Config {
        verbose: matches.is_present(name_verbose),
        launch_web: matches.is_present(name_launch_web),
        target_directory: value_t!(matches, name_target_directory, String)
            .unwrap_or(default_target_directory),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = get_program_input();

    // let cocoon = Cocoon::new(b"password");
    // let mut file = File::create("foo.cocoon")?;
    // encrypt_file(&cocoon, &mut file, "data".as_bytes().to_vec());

    // let mut unencrypted_file = File::create("foo.txt")?;
    // decrypt_file(&cocoon, &mut unencrypted_file);

    if config.launch_web {
        rocket::ignite()
             .mount("/node_modules", StaticFiles::from("node_modules"))
            .mount("/static", StaticFiles::from("static"))
            .mount("/", routes![index, files, new])
            .manage(config)
            .attach(Template::fairing())
            .launch();
    }
    Ok(())
}

#[post("/document", data = "<doc>")]
fn new(doc: Form<Document>) -> Result<(), Box<dyn Error>> {
    let cocoon = Cocoon::new(b"password");
    let mut file = File::create(format!(
        "static/{}_{}_{}_{}.cocoon",
        doc.date, doc.institution, doc.name, doc.page
    ))?;

    let mut f = File::open(format!("static/{}",&doc.orig_name))?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    encrypt_file(&cocoon, &mut file, buffer)?;
    Ok(())
}

fn encrypt_file(
    cocoon: &Cocoon<ThreadRng, Creation>,
    file: &mut File,
    data: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    cocoon.dump(data, file).unwrap();
    Ok(())
}

fn decrypt_file(
    cocoon: &Cocoon<ThreadRng, Creation>,
    file: &mut File,
) -> Result<(), Box<dyn Error>> {
    let data = cocoon.parse(file).unwrap();
    Ok(())
}

fn list_files(directory: &PathBuf) -> Vec<PathBuf> {
    if !directory.exists() {
        return Vec::new(); // TODO: turn this into an optional
    }
    env::set_current_dir(directory).unwrap();
    chain(
        glob("*.pdf").expect("Can't read directory."),
        glob("*.jpg").expect("Can't read directory."),
    )
    .map(|e| e.unwrap().into())
    .collect()
}

#[get("/")]
fn index() -> Template {
    let now: DateTime<Utc> = Utc::now();
    let mut context = HashMap::new();
    context.insert("filename".to_string(), "uboot.pdf".to_string());
    context.insert("date".to_string(),  now.format("%Y-%m-%d").to_string());
    Template::render("index", &context)
}

#[get("/files")]
fn files(config: State<Config>) -> JsonValue {
    let files: Vec<String> = list_files(&PathBuf::from(&config.target_directory))
        .iter()
        .map(|x| x.to_str().unwrap().to_owned())
        .collect();
    JsonValue(serde_json::json!(files))
}
