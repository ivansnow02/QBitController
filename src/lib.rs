use reqwest::header::{self, COOKIE};
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
extern crate clap;

#[derive(Deserialize, Debug)]
pub struct TorrentInfo {
    hash: String,
    save_path: String,
    name: String,
}

pub struct Config {
    pub qbittorrent_url: String,
    pub username: String,
    pub password: String,
    pub target_folder: String,
    pub action: String,
    client: Client,
    cookie: String,
}

impl Config {
    pub fn new(qbittorrent_url: String, username: String, password: String, target_folder: String, action: String) -> Self {
        Config {
            qbittorrent_url,
            username,
            password,
            target_folder,
            action,
            client: Client::new(),
            cookie: String::new(),
        }
    }
}
pub async fn send_action(config: &Config, torrents_res :Vec<TorrentInfo>) -> Result<(), Box<dyn Error>> {
    let mut torrent_hashes = Vec::new();

    for torrent in &torrents_res {
        if torrent.save_path == config.target_folder {
            torrent_hashes.push(torrent.hash.clone());
        }
    }

    if torrent_hashes.is_empty() {
        println!("no torrents found in the specified folder.");
        return Ok(());
    }

    let hashes = torrent_hashes.join("|");

    let url = format!("{}/api/v2/torrents/{}", config.qbittorrent_url, config.action);
    config.client
        .post(url)
        .header(COOKIE, config.cookie.clone())
        .form(&[("hashes", hashes)])
        .send()
        .await?;

    // print the name of the torrents that were paused or resumed

    for torrent in &torrents_res {
        if torrent_hashes.contains(&torrent.hash) {
            println!("{}: {}", config.action, torrent.name);
        }
    }

    Ok(())
}


pub async fn login(config: &mut Config) -> Result<Vec<TorrentInfo>, Box<dyn Error>> {


    let login_url = format!("{}/api/v2/auth/login", config.qbittorrent_url);
    let res = config.client
        .post(&login_url)
        .form(&[("username", config.username.clone()), ("password", config.password.clone())])
        .send()
        .await?;

    let headers = res.headers();

    let cookies = headers.get(header::SET_COOKIE).ok_or("No cookies")?;
    let cookie = cookies.to_str()?.to_string();

    config.cookie = cookie;

    // println!("Cookie: {}", cookie);

    let torrents_url = format!("{}/api/v2/torrents/info", config.qbittorrent_url);
    let torrent_res = config.client
        .get(&torrents_url)
        .header(COOKIE, &config.cookie)
        .send()
        .await?
        .json::<Vec<TorrentInfo>>()
        .await?;

    Ok(torrent_res)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login() {
        let mut config = Config::new("http://localhost:8089".to_string(), "admin".to_string(), "adminadmin".to_string(), "/tmp".to_string(), "pause".to_string());
        let torrents_res = login(&mut config).await.unwrap();
        assert_eq!(torrents_res.len(), 0);
    }

    #[tokio::test]
    async fn test_send_action() {
        let mut config = Config::new("http://localhost:8089".to_string(), "admin".to_string(), "adminadmin".to_string(), "/tmp".to_string(), "pause".to_string());
        let torrents_res = login(&mut config).await.unwrap();
        send_action(&config, torrents_res).await.unwrap();
    }
}
