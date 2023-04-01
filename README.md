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

### Logs in tests

To see all the logs coming out of a certain test case to debug you can use. Note we are using `bunyan` to prettify logs.

```
TEST_LOG=true cargo test health_check_works | bunyan
```

## Building

To build a docker image tagged as "zero2prod" according to the recipe specified in `Dockerfile`

```
docker build --tag zero2prod --file Dockerfile .
```

## Build

```
docker build --tag zero2prod --file Dockerfile .
```

then launch with

```
docker run -p 8000:8000 zero2prod
```
