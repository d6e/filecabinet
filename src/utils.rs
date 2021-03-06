use crate::Document;
use regex::Regex;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub struct OptDoc {
    pub(crate) date: Option<String>,
    pub(crate) institution: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) page: Option<String>,
}

/// Represents a Document with fields that were maybe parseable
impl OptDoc {
    pub fn new<T: AsRef<Path>>(filename: T) -> OptDoc {
        let filename = filename.as_ref();
        let filestem: &str = filename
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or(filename.to_str().unwrap());
        let v: Vec<&str> = filestem.split('_').collect();
        OptDoc {
            date: v.get(0).and_then(parse_date),
            institution: v.get(1).map(|x| x.to_string()),
            name: v.get(2).map(|x| x.to_string()),
            page: v.get(3).and_then(parse_page),
        }
    }
    pub fn is_parseable(&self) -> bool {
        self.date.is_some()
            && self.institution.is_some()
            && self.name.is_some()
            && self.page.is_some()
    }
}

pub fn is_normalized<P: AsRef<Path>>(source: P) -> bool {
    let source = source.as_ref();
    let extension: String = source
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or(String::new());
    let doc = OptDoc::new(source);
    if !doc.is_parseable() {
        return false;
    }
    match source.parent() {
        Some(basename) => {
            let target = basename.join(format!(
                "{}_{}_{}_{}.{}",
                doc.date.expect("date error"),
                doc.institution.expect("institution error"),
                doc.name.expect("name error"),
                doc.page.unwrap_or("1".to_owned()),
                extension
            ));
            source == target.as_path()
        }
        None => false,
    }
}

pub fn read_docs(path: &str) -> Vec<Document> {
    let dir_path = Path::new(&path).to_path_buf();
    list_files(&dir_path)
        .iter()
        .map(|path| {
            let mut full_path = dir_path.clone();
            full_path.push(path);
            Document::new(full_path.to_str().unwrap().to_string())
        })
        .collect()
}

pub fn extension<P: AsRef<Path>>(source: P) -> String {
    source
        .as_ref()
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or(String::new())
}

// TODO: use async paths
pub fn list_files(path: &PathBuf) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }
    path.read_dir()
        .expect("read_dir call failed")
        .map(|x| x.unwrap().path())
        .filter(|x| Path::new(x).is_file())
        .filter(|x| {
            let ext: String = x
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or(String::new());
            ext == "pdf" || ext == "jpg" || ext == "png" || ext == "cocoon"
        })
        .map(|x| x.file_name().unwrap().to_str().unwrap().to_owned())
        .collect()
}

pub fn to_camelcase(text: &str) -> String {
    let text = text.trim();
    let mut result = String::with_capacity(text.len());
    let mut start_of_word = true;
    for c in text.chars() {
        if c == ' ' {
            start_of_word = true;
        } else if start_of_word {
            result.push(c.to_ascii_uppercase());
            start_of_word = false;
        } else {
            result.push(c);
        }
    }
    result
}

#[test]
fn test_to_camelcase() {
    assert_eq!(to_camelcase("hello this is a test"), "HelloThisIsATest");
    assert_eq!(to_camelcase("_a"), "_a");
    assert_eq!(to_camelcase("boopLoop"), "BoopLoop");
}

lazy_static! {
    static ref RE_PARSE_PAGE: Regex = Regex::new(r"(\d+)").unwrap();
}

fn parse_page(text: &&str) -> Option<String> {
    RE_PARSE_PAGE
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_owned())
}

#[test]
fn test_parse_page() {
    assert_eq!(parse_page(&""), None);
    assert_eq!(parse_page(&"pg"), None);
    assert_eq!(parse_page(&"01"), Some("01".to_owned()));
    assert_eq!(parse_page(&"20"), Some("20".to_owned()));
    assert_eq!(parse_page(&"pg20"), Some("20".to_owned()));
}

lazy_static! {
    static ref RE_WITH_HYPHENS: Regex =
        Regex::new(r"^(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})").unwrap();
    static ref RE_NO_HYPHENS: Regex =
        Regex::new(r"^(?P<year>\d{4})(?P<month>\d{2})(?P<day>\d{2})").unwrap();
    static ref RE_YEAR_ONLY: Regex = Regex::new(r"^(?P<year>\d{4})").unwrap();
}

pub fn parse_date(text: &&str) -> Option<String> {
    // Returns the parsed date in ISO8601 format
    RE_WITH_HYPHENS
        .captures(text)
        .map(|x| {
            format!(
                "{}-{}-{}",
                x.name("year").unwrap().as_str(),
                x.name("month").unwrap().as_str(),
                x.name("day").unwrap().as_str(),
            )
        })
        .or(RE_NO_HYPHENS.captures(text).map(|x| {
            format!(
                "{}-{}-{}",
                x.name("year").unwrap().as_str(),
                x.name("month").unwrap().as_str(),
                x.name("day").unwrap().as_str(),
            )
        }))
        .or(RE_YEAR_ONLY.captures(text).map(|x| {
            format!(
                "{}-{}-{}",
                x.name("year").unwrap().as_str(),
                x.name("month").map(|m| m.as_str()).unwrap_or("01"),
                x.name("day").map(|m| m.as_str()).unwrap_or("01"),
            )
        }))
}

#[test]
fn test_parse_date_hyphens() {
    assert_eq!(
        parse_date(&"2020-04-03_boop_loop"),
        Some("2020-04-03".to_string())
    )
}

#[test]
fn test_parse_date_no_hyphens() {
    assert_eq!(
        parse_date(&"20180530_boop_loop"),
        Some("2018-05-30".to_string())
    )
}
#[test]
fn test_parse_date_year_only() {
    assert_eq!(
        parse_date(&"2018_boop_loop"),
        Some("2018-01-01".to_string())
    )
}
