use clap::{App, Arg};
use scraper::{Html, Selector};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::process::exit;
use std::thread::sleep;
use std::time;

fn main() {
    let matches = App::new("Pixiv novel downloader")
        .version("0.1.0")
        .author("Senventise")
        .about("Download novels from pixiv.net")
        .arg(Arg::with_name("URL").required(true).index(1))
        .arg(
            Arg::with_name("TYPE")
                .short("t")
                .long("type")
                .help("Type of the novel: single/series")
                .default_value("single"),
        )
        .get_matches();
    let url = matches.value_of("URL").unwrap();
    match matches.value_of("TYPE").unwrap() {
        "single" => {
            download_single(url, "");
        }
        "series" => download_series(url),
        _ => (),
    }
}

fn get_html(url: &str) -> String {
    let response = reqwest::blocking::get(url);
    match response {
        Ok(resp) => resp.text().unwrap(),
        Err(error) => {
            println!("{:?}", error.source().unwrap());
            exit(0);
        }
    }
}

fn download_single(url: &str, redirect_to: &str) {
    let fragment = Html::parse_fragment(&get_html(url));
    let selector = Selector::parse("#meta-preload-data").unwrap();
    let mut content = "";
    for element in fragment.select(&selector) {
        content = element.value().attr("content").unwrap();
    }
    let j = json::parse(content).unwrap();
    if j["novel"].is_empty() {
        panic!();
    }
    for (_, novel) in j["novel"].entries() {
        let title = novel["title"].to_string().replace(" ", "_");
        println!("Downloading: {}", title);
        let author = novel["userName"].to_string();
        if redirect_to.is_empty() {
            let mut file =
                File::create(format!("{}-{}.txt", title, author)).expect("Failed to create file.");
            file.write_all(novel["content"].to_string().replace("\n\n", "\n").as_ref())
                .expect("Failed to write.");
        } else {
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(format!("{}-{}.txt", redirect_to, author))
                .expect("Failed to create file.");
            file.write_all(format!("《{}》\n", title).as_ref())
                .expect("Failed to write.");
            file.write_all((novel["content"].to_string().replace("\n\n", "\n") + "\n").as_ref())
                .expect("Failed to write.");
        }
    }
}

fn download_series(url: &str) {
    let series_id: Vec<&str> = url.split('/').collect();
    let series_id = series_id.last().unwrap();
    let api_url = format!("https://www.pixiv.net/ajax/novel/series/{}", series_id);
    let api_resp = json::parse(&get_html(&api_url)).unwrap();
    let title = &api_resp["body"]["title"].to_string();
    let api_url = format!(
        "https://www.pixiv.net/ajax/novel/series_content/{}?limit=30&last_order=0&order_by=asc",
        series_id
    );
    let api_resp = json::parse(&get_html(&api_url)).unwrap();
    for novel in api_resp["body"]["seriesContents"].members() {
        let novel_url = format!("https://www.pixiv.net/novel/show.php?id={}", novel["id"]);
        download_single(&novel_url, title);
        sleep(time::Duration::from_secs(1));
    }
}
