# cargo-chef builds project dependencies for optimal layer caching
FROM lukemathwalker/cargo-chef:latest-rust-1.68.2-slim AS chef
# Lets switch working directory to `app` (equivalent to `cd app`)
# The app folder will be created for us by Docker in case it does not exist already
WORKDIR /app
# Install the required dependencies for our linking configuration
RUN apt update && apt install lld clang -y

FROM chef AS planner
# Copy all the files from our working directory environment to our Docker image
COPY . . 
# Compute a lock like filefor our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build the project dependencies, not our application
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point iof our dependency tree stays the same,
# all layers should be cached
COPY . .
# Forces sqlx to look at saved metadata (sqlx-data.json)
# instead of trying to query a live database at compile time
ENV SQLX_OFFLINE true
# Build our project
# well use the release profile to make it very fast
RUN cargo build --release --bin zero2prod

FROM debian:bullseye-slim AS runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TSL certificates
# when establishing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# Copy the compiled binary from the builder environment
# to our runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod
# We need the configuration file at runtime!
COPY configuration configuration
# Instruct binary in Docker image to use the production config
ENV APP_ENVIRONMENT production
# When `docker run` is executed, launch the binary!
ENTRYPOINT [ "./zero2prod" ]