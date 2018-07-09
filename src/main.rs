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

fn ping(cfg: &[ConfigEntry]) -> usize {
    let mut processed: usize = 0;

    for e in cfg.iter() {
        let server = e.server.as_str();
        let user = e.user.as_str();
        let password = e.password.as_str();

        let addr = match split_host_port(server) {
            Ok((host, port)) => {
                if let Ok(port) = port.parse::<u16>() {
                    (host, port)
                } else {
                    error!("invalid port format: {}", server);
                    continue;
                }
            }
            Err(err) => {
                error!("invalid server '{}' format: {}", server, err);
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

        if let Err(err) = conn.noop() {
            error!("'noop' command failed: {}", err);
            continue;
        };

        if let Err(err) = conn.logout() {
            error!("logout error for {}: {}", user, err);
            continue;
        }

        processed += 1;
    }

    processed
}

fn split_host_port(hostport: &str) -> Result<(&str, &str), &'static str> {
    match hostport.rfind(':') {
        Some(pos) => {
            if hostport.chars().nth(0) == Some('[') {
                match hostport.rfind(']') {
                    Some(end) if end + 1 == hostport.len() => Err("missing port"),
                    Some(end) if end + 1 == pos => Ok((&hostport[1..end], &hostport[end + 2..])),
                    Some(end) => if hostport.chars().nth(end + 1) == Some(':') {
                        Err("too many colons in address")
                    } else {
                        Err("missing port")
                    },
                    None => Err("missing ']' in address"),
                }
            } else {
                let host = &hostport[0..pos];
                match host.find(':') {
                    None => Ok((host, &hostport[pos + 1..])),
                    _ => Err("too many colons in address"),
                }
            }
        }
        None => Err("missing port in address"),
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
