# zero2prod

zero to production in rust

## To start

From root run

```
./scripts/init_db.sh
```

to get database migration started
or, if docker ps returns postgres container already, run

```
SKIP_DOCKER=true ./scripts/init_db.sh
```

then run

```
cargo watch -x check -x test -x fmt -x run
```
