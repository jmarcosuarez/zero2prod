use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Panic of we cant read configuration
    let configuration = get_configuration().expect("Failed to read configuration");
    // We have removed the hard-coded '8000' - its coming from Settings
    let address = format!("127.0.01:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on out Server
    run(listener)?.await
}
