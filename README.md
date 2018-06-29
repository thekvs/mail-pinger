To run with debug logging: `RUST_LOG=DEBUG ./target/debug/mail-pinger -c /path/to/config.yaml`

## Configuration file example

```yaml
- server: imap.mail.ru:993
  user: vasya@mail.ru
  password: fsdsf
- server: imap.yandex.ru:993
  user: petya@yandex.ru
  password: fsdfsfsf

```
