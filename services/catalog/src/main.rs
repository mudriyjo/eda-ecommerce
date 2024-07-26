use axum::{response::Html, routing::get, Extension, Json};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
struct Env {
    pub(crate) server_address: String,
    pub(crate) db_connection_string: String
}

async fn get_env_variables() -> anyhow::Result<Env> {
    let server = std::env::var("SERVER");
    let db_connection_string = std::env::var("DATABASE_URL");
    // Read .env only if in local env some of variable is empty
    if server.is_err() || db_connection_string.is_err() {
        dotenv::dotenv().expect("Can't find .env file or variables and can't load them");
    }
    let server_address = std::env::var("SERVER")?;
    let db_connection_string = std::env::var("DATABASE_URL")?;
    Ok(Env{server_address, db_connection_string})
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    
    let env = get_env_variables().await?;
    let pool = PgPool::connect(&env.db_connection_string).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    color_eyre::install().expect("Error with starting color eyre hook...");

    tracing_subscriber::fmt::init();

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
