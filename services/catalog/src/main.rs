use std::collections::HashMap;

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
#[derive(Debug)]
struct Env {
    pub(crate) server_address: String,
    pub(crate) db_connection_string: String,
    pub(crate) kafka_group_id: String,
    pub(crate) kafka_broker: String,
}

impl Env {
    fn list_of_variables() -> HashMap<String, Option<String>> {
        vec![
            ("SERVER".to_string(), None),
            ("DATABASE_URL".to_string(), None),
            ("KAFKA_GROUP_ID".to_string(), None),
            ("KAFKA_BROKER".to_string(), None),
        ].into_iter().collect()
    }
    
    fn not_all_variables_exist(env_variables: &HashMap<String, Option<String>>) -> bool {
        env_variables.iter().any(|entry| entry.1.is_none())
    }

    fn read_all_variables(env_variables: &mut HashMap<String, Option<String>>) {
        for entry in env_variables.iter_mut() {
            if let Ok(variable) = std::env::var(entry.0) {
                *entry.1 = Some(variable)
            }
        }
    }

    pub fn new() -> anyhow::Result<Self> {
        let mut variables = Self::list_of_variables();
        
        // Read from env 
        Self::read_all_variables(&mut variables);

        if Self::not_all_variables_exist(&variables) {
            dotenv::dotenv().expect("Can't find .env file or variables and can't load them");
        }
        
        // Read after read from .env file
        Self::read_all_variables(&mut variables);

        if Self::not_all_variables_exist(&variables) {
            let var_string = variables.iter_mut().fold("".to_string(), |mut acc, el| {
                acc.push_str(format!("{} : {:?} \n", el.0, el.1).as_str());
                acc
            });
            anyhow::bail!(format!("Not all variables exist: {}", var_string))
        } else {
            Ok(Env{
                server_address: variables.get("SERVER").unwrap().clone().unwrap(),
                db_connection_string: variables.get("DATABASE_URL").unwrap().clone().unwrap(),
                kafka_group_id: variables.get("KAFKA_GROUP_ID").unwrap().clone().unwrap(),
                kafka_broker: variables.get("KAFKA_BROKER").unwrap().clone().unwrap()
            })
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::new()?;

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
