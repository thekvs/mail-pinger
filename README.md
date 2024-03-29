## About

Keep not frequently used email accounts alive by periodically using mailboxes through IMAP protocol.

Some public email services will close your account if you don't use it for a long time. This simple program allows you to simulate some
(fake) activity on your account via IMAP protocol.

## Usage

1. To run with debug logging enabled and custom configuration file: `RUST_LOG=DEBUG ./target/debug/mail-pinger -c /path/to/config.yaml`
1. To run with default (`$HOME/.config/mail-pinger/config.yaml`) configuration file: `./target/debug/mail-pinger`

### Configuration file example

```yaml
- server: imap.mail.ru:993
  user: vasya@mail.ru
  password: coolpassword1
- server: imap.yandex.ru:993
  user: petya@yandex.ru
  password: anothercollpassword

```

## Compiling

You need to have Rust ecosystem installed.

* `cargo build` or `cargo build --release` to build a binary.
* `make` to build a binary and `make deb` to package it in the `.deb` format (you'll need docker installed).
