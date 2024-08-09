use reqwest::header::{self, COOKIE};
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
extern crate clap;
use clap::{Arg, Command};

#[derive(Deserialize, Debug)]
struct TorrentInfo {
    hash: String,
    save_path: String,
    name: String,
}

async fn send_action(
    action: &str,
    qbittorrent_url: &str,
    cookie: &str,
    client: &Client,
    torrents_res: Vec<TorrentInfo>,
    target_folder: &str,
) -> Result<(), Box<dyn Error>> {
    let mut torrent_hashes = Vec::new();

    for torrent in &torrents_res {
        if torrent.save_path == target_folder {
            torrent_hashes.push(torrent.hash.clone());
        }
    }

    if torrent_hashes.is_empty() {
        println!("no torrents found in the specified folder.");
        return Ok(());
    }

    let hashes = torrent_hashes.join("|");

    let url = format!("{}/api/v2/torrents/{}", qbittorrent_url, action);
    client
        .post(url)
        .header(COOKIE, cookie)
        .form(&[("hashes", hashes)])
        .send()
        .await?;

    // print the name of the torrents that were paused or resumed

    for torrent in torrents_res {
        if torrent_hashes.contains(&torrent.hash) {
            println!("{}: {}", action, torrent.name);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("qBittorrent Controller")
        .version("1.0")
        .author("Ivan Snow")
        .about("Controls qBittorrent downloads based on folder path")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .help("qBittorrent Web UI URL"),
        )
        .arg(
            Arg::new("username")
                .short('n')
                .long("username")
                .required(true)
                .help("Username for qBittorrent Web UI"),
        )
        .arg(
            Arg::new("password")
                .short('p')
                .long("password")
                .help("Password for qBittorrent Web UI"),
        )
        .arg(
            Arg::new("target_folder")
                .short('t')
                .long("target-folder")
                .required(true)
                .help("Target folder path for torrent downloads"),
        )
        .arg(
            Arg::new("action")
                .short('a')
                .long("action")
                .required(true)
                .help("Action to perform: pause or resume"),
        )
        .get_matches();

    let qbittorrent_url = matches
        .get_one::<String>("url")
        .map(|s| s.as_str())
        .unwrap_or("http://localhost:8080");
    let username = matches
        .get_one::<String>("username")
        .map(|s| s.as_str())
        .expect("username cannot be empty");
    let password = matches
        .get_one::<String>("password")
        .map(|s| s.as_str())
        .expect("password cannot be empty");
    let target_folder = matches
        .get_one::<String>("target_folder")
        .map(|s| s.as_str())
        .expect("target folder cannot be empty");
    let action = matches
        .get_one::<String>("action")
        .map(|s| s.as_str())
        .expect("action cannot be empty");

    let client = Client::new();

    let login_url = format!("{}/api/v2/auth/login", qbittorrent_url);
    let res = client
        .post(&login_url)
        .form(&[("username", username), ("password", password)])
        .send()
        .await?;

    let headers = res.headers();

    let cookies = headers.get(header::SET_COOKIE).ok_or("No cookies")?;
    let cookie = cookies.to_str()?.to_string();

    // println!("Cookie: {}", cookie);

    let torrents_url = format!("{}/api/v2/torrents/info", qbittorrent_url);
    let torrents_res = client
        .get(&torrents_url)
        .header(COOKIE, &cookie)
        .send()
        .await?
        .json::<Vec<TorrentInfo>>()
        .await?;

    // println!("{:?}", torrents_res);

    send_action(action, qbittorrent_url, &cookie, &client, torrents_res, target_folder).await?;

    Ok(())
}
