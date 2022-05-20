extern crate core;

mod pixiv;

use crate::pixiv::{Novel, Series};
use clap::{App, Arg};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, USER_AGENT};
use sanitize_filename::sanitize;
use std::env;
use std::path::Path;
use std::thread::sleep;
use std::time;

fn get_cookie_from_env() -> String {
    match env::var("PIXIV_COOKIE") {
        Ok(cookie) => cookie,
        Err(_) => panic!("Environment varaible PIXIV_COOKIE not set"),
    }
}

fn get_response(url: &str, with_cookie: bool) -> String {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.41 Safari/537.36"));
    if with_cookie {
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&get_cookie_from_env()).unwrap(),
        );
    }
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let response = client.get(url).send();

    match response {
        Ok(resp) => resp.text().unwrap(),
        Err(error) => {
            println!("{:?}", error);
            panic!("Request failed.");
        }
    }
}

fn get_novel_content(url: &str) -> Novel {
    let novel_id = url.rsplit_once('=').unwrap().1.to_string(); // pid
    let api_url = format!("https://www.pixiv.net/ajax/novel/{}", novel_id);
    let api_resp = json::parse(&get_response(&api_url, false)).unwrap();
    let title = api_resp["body"]["title"].to_string();
    let author = api_resp["body"]["userName"].to_string();
    let content = api_resp["body"]["content"].to_string();
    let uid = api_resp["body"]["userId"].to_string();

    Novel::new(title, author, content, novel_id, uid)
}

fn download_single(url: &str) {
    let novel = get_novel_content(url);
    novel.save();
}

fn download_series(url: &str) {
    let series_id = url.rsplit_once('/').unwrap().1.to_string();
    let api_url = format!("https://www.pixiv.net/ajax/novel/series/{}", series_id);
    let api_resp = json::parse(&get_response(&api_url, false)).unwrap();
    let title = api_resp["body"]["title"].to_string();
    let author = api_resp["body"]["userName"].to_string();
    let uid = api_resp["body"]["userId"].to_string();
    let total = api_resp["body"]["total"]
        .as_u8()
        .expect("Failed to parse json.");
    println!("Downloading series:《{}》", title);
    let mut series = Series::new(title, author, series_id, uid);
    let mut last_order = 0u8;
    loop {
        // iterate by pages
        let api_url = format!(
            "https://www.pixiv.net/ajax/novel/series_content/{}?limit=30&last_order={}&order_by=asc",
            &series.pid, last_order
        );
        let api_resp = json::parse(&get_response(&api_url, false)).unwrap();
        for novel_json in api_resp["body"]["seriesContents"].members() {
            print!("[{}/{}]", &last_order + 1, &&total);
            let novel_url = format!(
                "https://www.pixiv.net/novel/show.php?id={}",
                novel_json["id"]
            );
            let novel = get_novel_content(&novel_url);
            series.append(novel);
            sleep(time::Duration::from_secs(2)); // prevent potential restriction
            last_order = novel_json["series"]["contentOrder"]
                .as_u8()
                .expect("Failed to parse contentOrder");
        }

        if last_order == total {
            break;
        }
    }
}

fn download_user_bookmarks(uid: &str) {
    println!("User: {}", uid);
    let mut offset = 0u8;
    let mut iterated = 0u8;
    loop {
        let api_url = format!(
            "https://www.pixiv.net/ajax/user/{}/novels/bookmarks?tag=&offset={}&limit=24&rest=show&lang=zh",
            uid, offset
        );
        let api_resp = json::parse(&get_response(&api_url, true)).unwrap();
        for novel_json in api_resp["body"]["works"].members() {
            if novel_json["isMasked"].as_bool().unwrap() {
                println!("[DELETED]: {}", novel_json["id"]);
                continue;
            }
            let filename = sanitize(format!(
                "{}-{}.txt",
                novel_json["title"], novel_json["userName"]
            ));
            let path = Path::new(&filename);
            if path.exists() {
                println!("{} exists, skipped.", filename);
                continue;
            }
            download_single(&format!(
                "https://www.pixiv.net/novel/show.php?id={}",
                novel_json["id"]
            ));
            iterated += 1;
            sleep(time::Duration::from_secs(3))
        }
        offset += 24;
        if iterated == api_resp["body"]["total"].as_u8().unwrap() {
            break;
        }
        sleep(time::Duration::from_secs(5));
    }
}

fn main() {
    let matches = App::new("Pixiv novel downloader")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Senventise")
        .about("Download novels from pixiv.net")
        .arg(Arg::with_name("URL").required(true))
        .arg_from_usage("-b, --bookmark 'Download user's bookmarks instead of works'")
        .get_matches();
    let url = matches.value_of("URL").unwrap();
    if url.starts_with("https://www.pixiv.net/users/") {
        // user's bookmark / works
        let re = Regex::new(r"\d+").unwrap();
        let uid = re.find(url).unwrap().as_str();
        if matches.occurrences_of("bookmark") > 0 {
            download_user_bookmarks(uid);
        }
    } else if url.contains("series") {
        download_series(url);
    } else {
        download_single(url);
    }
}
