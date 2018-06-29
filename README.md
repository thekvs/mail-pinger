## Usage

1. To run with debug logging enabled and custom configuration file: `RUST_LOG=DEBUG ./target/debug/mail-pinger -c /path/to/config.yaml`
1. To run with default (`$HOME/.config/mail-pinger/config.yaml`) configuration file: `./target/debug/mail-pinger`

## Configuration file example

```yaml
- server: imap.mail.ru:993
  user: vasya@mail.ru
  password: fsdsf
- server: imap.yandex.ru:993
  user: petya@yandex.ru
  password: fsdfsfsf

```
