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
extern crate threadpool;

use clap::{App, Arg};
use imap::client::Client;
use native_tls::TlsConnector;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    env, error::Error, fs::File, os::unix::fs::PermissionsExt, path::Path, process, sync::Arc,
};
use threadpool::ThreadPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigEntry {
    server: String,
    user: String,
    password: String,
}

fn read_config_file<P>(path: P) -> Result<Vec<ConfigEntry>, Box<Error>>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let cfg = serde_yaml::from_reader(file)?;

    Ok(cfg)
}

fn ping_single(e: &ConfigEntry) -> Result<(), Box<Error>> {
    let server = e.server.as_str();
    let user = e.user.as_str();
    let password = e.password.as_str();

    let (host, port) = split_host_port(server)?;
    let port = port.parse::<u16>()?;
    let addr = (host, port);

    let ssl_connector = TlsConnector::builder().unwrap().build().unwrap();
    let mut conn = Client::secure_connect(addr, addr.0, &ssl_connector)?;

    conn.login(user, password)?;
    conn.capabilities()?;
    conn.select("INBOX")?;
    conn.noop()?;
    conn.logout()?;

    Ok(())
}

fn ping_all(cfg: Vec<ConfigEntry>) -> Result<usize, Box<Error>> {
    let threads_num = 10;
    let pool = ThreadPool::new(threads_num);
    let processed = Arc::new(AtomicUsize::new(0));

    cfg.into_iter().for_each(|ref e| {
        let e = e.clone();
        let processed = processed.clone();
        pool.execute(move || match ping_single(&e) {
            Ok(()) => {
                let _ = processed.fetch_add(1, Ordering::SeqCst);
            }
            Err(e) => error!("{}", e),
        })
    });

    pool.join();

    Ok(processed.load(Ordering::SeqCst))
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

    let config_file = if matches.is_present("config") {
        matches.value_of("config").unwrap().to_string()
    } else {
        match env::var("HOME") {
            Ok(v) => format!("{}/.config/mail-pinger/config.yaml", v),
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
            let total = config.len();
            if let Ok(processed) = ping_all(config) {
                println!(
                    "succesfully processed {} entries out of {}",
                    processed, total
                );
            }
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
