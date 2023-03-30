# zero2prod

zero to production in rust

## To start

From root run

```
./scripts/init_db.sh
```

then run

```
cargo watch -x check -x test -x fmt -x run
```

## Logging

We default to print all logs at info level or above. As if we did run the app with the env variable `RUST_LOG` set to `info`. Eg. `RUST_LOG=info cargo run`.
Other values can be `trace`, `debug`, `info`, `warn` and `error`.
