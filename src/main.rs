use std::error::Error;
extern crate clap;
use clap::{Arg, Command};
use qbit_controller::{Config, send_action, login};


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

    let mut config = Config::new(qbittorrent_url.to_string(), username.to_string(), password.to_string(), target_folder.to_string(), action.to_string());

    let torrents_res = login(&mut config).await?;

    send_action(&config, torrents_res).await?;

    Ok(())
}
