To run with debug logging enabled: `RUST_LOG=DEBUG ./target/debug/mail-pinger -c /path/to/config.yaml`

## Configuration file example

```yaml
- server: pop.mail.ru:993
  user: vasya@mail.ru
  password: fsdsf
- server: pop.yandex.ru:993
  user: petya@yandex.ru
  password: fsdfsfsf

```