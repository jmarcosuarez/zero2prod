use sqlx::PgPool;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Redirect all `logs`'s events to out subscriber
    LogTracer::init().expect("Failed to set logger");

    // we are falling back to printing all spans at ifo-level or above
    // if the RUST_LOG environment variable has not been set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "zero2prod".into(),
        // output the formatted spans to stdout
        std::io::stdout,
    );
    // The `with` method is provided by `SubscriberExt` , an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    // `set_global_default` can be used by applications to specify
    // what subscriber should be used to process spans
    set_global_default(subscriber).expect("Failed to set subscriber");

    // Panic of we cant read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    // We have removed the hard-coded '8000' - its coming from Settings
    let address = format!("127.0.01:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on out Server
    run(listener, connection_pool)?.await
}
