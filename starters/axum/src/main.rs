use api::config::get_configuration;
use api::config::logging::{get_subscriber, init_subscriber};
use api::startup::build;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("api".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    build(configuration.clone()).await
}
