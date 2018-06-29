#[macro_use]
extern crate serde_derive;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate imap;
extern crate libc;
extern crate native_tls;
extern crate serde;
extern crate serde_yaml;

use clap::{App, Arg};
use imap::client::Client;
use native_tls::TlsConnector;
use std::{env, error::Error, fs::File, os::unix::fs::PermissionsExt, path::Path, process};

const DEFAULT_IMAP_PORT: u16 = 993;

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

fn ping(cfg: &Vec<ConfigEntry>) -> usize {
    let mut processed: usize = 0;

    for e in cfg.iter() {
        let server = e.server.as_str();
        let user = e.user.as_str();
        let password = e.password.as_str();

        let items: Vec<&str> = server.splitn(2, ':').collect();

        let addr = match items.len() {
            2 => {
                if let Ok(port) = items[1].parse::<u16>() {
                    (items[0], port)
                } else {
                    error!("invalid port format: {}", server);
                    continue;
                }
            }
            1 => (items[0], DEFAULT_IMAP_PORT),
            _ => {
                error!(
                    "invalid format for 'server' configuration entry: {}",
                    server
                );
                continue;
            }
        };

        let ssl_connector = TlsConnector::builder().unwrap().build().unwrap();
        let mut conn = match Client::secure_connect(addr, addr.0, &ssl_connector) {
            Ok(c) => c,
            Err(err) => {
                error!("couldn't connect to {}: {}", server, err);
                continue;
            }
        };

        if let Err(err) = conn.login(user, password) {
            error!("login error for {}: {:?}", user, err);
            continue;
        }

        match conn.capabilities() {
            Ok(capabilities) => {
                for capability in capabilities.iter() {
                    debug!("{}", capability);
                }
            }
            Err(e) => {
                error!("error parsing capabilities: {}", e);
                continue;
            }
        };

        match conn.select("INBOX") {
            Ok(mailbox) => {
                debug!("selected INBOX. {}", mailbox);
            }
            Err(e) => {
                error!("error selecting INBOX: {}", e);
                continue;
            }
        };

        match conn.noop() {
            Err(err) => {
                error!("'noop' command failed: {}", err);
                continue;
            }
            _ => (),
        };

        match conn.logout() {
            Err(err) => {
                error!("logout error for {}: {}", user, err);
                continue;
            }
            _ => (),
        }

        processed += 1;
    }

    processed
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
                error!("error getting env. variable $HOME: {:?}", e);
                process::exit(-1);
            }
        }
    };

    match std::fs::metadata(&config_file) {
        Ok(meta) => {
            if meta.permissions().mode()
                & (libc::S_IRGRP
                    | libc::S_IWGRP
                    | libc::S_IXGRP
                    | libc::S_IROTH
                    | libc::S_IWOTH
                    | libc::S_IXOTH) > 0
            {
                error!(
                    "config file '{}' has invalid permission, must be '-rw-------'",
                    config_file
                );
                process::exit(-1);
            };
        }
        Err(err) => {
            error!(
                "couldn't get config file's '{}' metadata: {}",
                config_file, err
            );
            process::exit(-1);
        }
    };

    match read_config_file(config_file.as_str()) {
        Ok(config) => {
            let processed = ping(&config);
            println!(
                "succesfully processed {} entries out of {}",
                processed,
                config.len()
            );
        }
        Err(err) => {
            error!(
                "error occured while reading '{}' configuration file: {}",
                config_file, err
            );
            process::exit(-1);
        }
    }
}
