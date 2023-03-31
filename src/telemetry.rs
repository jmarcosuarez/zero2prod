use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Compose multiple layers into a `tracing`'s subscriber
///
/// # Implementation notes
///
/// We are using `impl Subscriber ` as a return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed very complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    // AThis `weird` syntax is a higer-rranked-trait-bound (HRTB)
    // It basically means that Sink implements the `MakeWriter1
    // trait for all choises of the lifetime `'a`
    // Check https://doc.rust-lang.org/nomicon/hrtb.html for more details
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // we are falling back to printing all spans at info-level or above
    // if the RUST_LOG environment variable has not been set
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    // The `with` method is provided by `SubscriberExt` , an extension
    // trait for `Subscriber` exposed by `tracing_subscriber`
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global to process span data.
///
/// It should only be called once
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all `logs`'s events to out subscriber
    LogTracer::init().expect("Failed to set logger");
    // `set_global_default` can be used by applications to specify
    // what subscriber should be used to process spans
    set_global_default(subscriber).expect("Failed to set subscriber");
}
