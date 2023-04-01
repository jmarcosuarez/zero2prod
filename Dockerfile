# We use the latest Rust stable release as base image
FROM rust:1.68.1

# Lets switch working directory to `app` (equivalent to `cd app`)
# The app folder will be created for us by Docker in case it does not exist already
WORKDIR /app
# Install the required dependencies for our linking configuration
RUN apt update && apt install lld clang -y
# Copy all the files from our working directory environment to our Docker image
COPY . . 
# Forces sqlx to look at saved metadata (sqlx-data.json)
# instead of trying to query a live database at compile time
ENV SQLX_OFFLINE true
# Lets build out binary!
# well use the release profile to make it very fast
RUN cargo build --release
# Instruct binary in Docker image to use the production config
ENV APP_ENVIRONMENT production
# When `docker run` is executed, launch the binary!
ENTRYPOINT [ "./target/release/zero2prod" ]