# zero2prod

zero to production in rust

### About

Subscription flow with proper confirmation email:

- Every time an user wants to subscribe to our newsletter they fire a POST request to `/subscriptions`. Our request handler will:

1. Add their details to our DB in the `subscriptions` table, with status equal to `pending_confirmation`.
2. Generate unique `subscription_token`.
3. Store this `subscription-token` in DB against their id in a `subscription_tokens` table.
4. Send an email to the new subscriber containing a link structured as `https://<api.domain>/subscriptions/confirm?token=<subscription_token>`.
5. Return a `200 OK`.

Once they click on the link, a browser tab will open and a new GET request will be fired to out GET `/subscriptions/confirm` endpoint. Our request handler will:

1. Retrieve `subscription_token` from query parameters.
2. Retrieve the `subscriber_id` associated with `subscription_token` from the `subscription_tokens` table.
3. Update the subscriber status from `pending_confirmation` to `active` in the `subscriptions` table.
4. Return a `200 Ok`.

## To start

From root run

```
./scripts/init_db.sh
```

then run:

```
./scripts/init_redis.sh
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

## Build

To build a docker image tagged as "zero2prod" according to the recipe specified in `Dockerfile`

```
docker build --tag zero2prod --file Dockerfile .
```

then launch with

```
docker run -p 8000:8000 zero2prod
```

## Migrations

- To create new table/column/etc in DB create migration with: `sqlx migrate add create_users_table.sql`.
- Add SQL to the created file under `migrations` folder.
- then run `SKIP_DOCKER=true ./scripts/init_db.sh`.
- then run `cargo sqlx prepare` just so tests can pass on CI.

## Credentials

To log into the user dashboard you can use:

- username: _admin_
- password: _everythinghastostartsomewhere_
