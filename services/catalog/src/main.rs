use axum::{response::Html, routing::get, Extension, Json};
use env::Env;
use futures::StreamExt;
use rdkafka::{consumer::{Consumer, StreamConsumer}, ClientConfig};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod env;

async fn index() -> Html<String> {
    Html("<h1>Hello world!</h1>".to_string())
}

#[derive(OpenApi)]
#[openapi(paths(openapi))]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/api-docs/openapi.json",
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn listening_kafka_topic(kafka_consumer: &StreamConsumer) {
    kafka_consumer.stream().for_each(|msg| {
        async move {
            println!("{:?}", msg);
        }
    }).await;
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Read env variables.");
    let env = Env::new()?;

    tracing::info!("Connection to PostgreSQL.");
    let pool = PgPool::connect(&env.db_connection_string).await?;
    
    tracing::info!("Connection to Kafka.");
    let kafka_consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", env.kafka_group_id)
        .set("bootstrap.servers", env.kafka_broker)
        .set("session.timeout.ms", "6000")
        .create()?;

    kafka_consumer.subscribe(&["example"])?;
    // listening_kafka_topic(&kafka_consumer).await;

    tracing::info!("Execute migrations.");
    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::info!("Set uping hook for panic.");
    color_eyre::install().expect("Error with starting color eyre hook...");

    let extension_pool = pool.clone();
    let connection = tokio::net::TcpListener::bind(env.server_address).await?;

    let router = axum::Router::new()
        .route("/", get(index))
        .route("/swagger/openapi.json", get(openapi))
        .merge(SwaggerUi::new("/swagger-ui"))
        .layer(Extension(extension_pool));

    tracing::info!("Start server...");
    axum::serve(connection, router).await?;

    Ok(())
}
