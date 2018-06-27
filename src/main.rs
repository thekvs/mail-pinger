#[macro_use]
extern crate serde_derive;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate imap;
extern crate native_tls;
extern crate serde;
extern crate serde_yaml;

use clap::{App, Arg};
use imap::client::Client;
use native_tls::TlsConnector;
use std::{env, error::Error, fs::File, path::Path, process};

#[derive(Debug, Serialize, Deserialize)]
struct ConfigEntry {
    server: String,
    user: String,
    password: String,
}

fn read_config_file<P: AsRef<Path>>(path: P) -> Result<Vec<ConfigEntry>, Box<Error>> {
    let file = File::open(path)?;
    let cfg = serde_yaml::from_reader(file)?;

    Ok(cfg)
}

fn mail_stat(cfg: &Vec<ConfigEntry>) {
    for e in cfg.iter() {
        let server = e.server.as_str();
        let user = e.user.as_str();
        let password = e.password.as_str();

        let items: Vec<&str> = server.splitn(2, ':').collect();

        let addr = match items.len() {
            2 => (items[0], items[1].parse::<u16>().unwrap()),
            1 => (items[0], 993),
            _ => {
                error!("invalid format for 'server' configuration entry: {}", server);
                process::exit(-1);
            }
        };

        let ssl_connector = TlsConnector::builder().unwrap().build().unwrap();
        let mut conn = match Client::secure_connect(addr, addr.0, &ssl_connector) {
            Ok(c) => c,
            Err(err) => {
                error!("couldn't connect to {}: {}", server, err);
                process::exit(-1);
            }
        };

        if let Err(err) = conn.login(user, password) {
            error!("login error for {}: {:?}", user, err);
            process::exit(-1);
        }

        match conn.capabilities() {
            Ok(capabilities) => {
                for capability in capabilities.iter() {
                    debug!("{}", capability);
                }
            }
            Err(e) => {
                error!("Error parsing capabilities: {}", e);
                process::exit(-1);
            }
        };

        match conn.select("INBOX") {
            Ok(mailbox) => {
                debug!("{}", mailbox);
            }
            Err(e) => {
                error!("Error selecting INBOX: {}", e);
                process::exit(-1);
            }
        };

        conn.logout().unwrap();
    }
}

fn main() {
    env_logger::init();

    let matches = App::new("mail pinger")
        .version("0.1.0")
        .author("Konstantin Sorokin <kvs@sigterm.ru>")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("path to the configuration file"),
        )
        .get_matches();

    let config_file: String;

    if matches.is_present("config") {
        config_file = matches.value_of("config").unwrap().to_string();
    } else {
        match env::var("HOME") {
            Ok(v) => config_file = format!("{}/.config/mail-pinger/config.yaml", v),
            Err(e) => {
                error!("Error: {:?}", e);
                process::exit(-1);
            }
        }
    };

    match read_config_file(config_file.as_str()) {
        Ok(config) => mail_stat(&config),
        Err(err) => {
            error!(
                "Error occured while reading '{}' configuration file: {}",
                config_file, err
            );
            process::exit(-1);
        }
    }
}
