use axum::Router;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::tool;
use rmcp::tool_handler;
use rmcp::tool_router;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::ErrorData as McpError;
use rmcp::Json;
use rmcp::ServerHandler;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpListener;

#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub struct EchoInput {
    message: String,
    nested_item: NestedItem,
}

#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub struct NestedItem {
    message: String,
}

#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub struct EchoOutput {
    message: String,
}

#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub struct EchoOneOf {
    one_of: OneOf,
}

#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub enum OneOf {
    Hello(Hello),
    World(World),
}

#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub struct Hello {
    message: String,
}

#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub struct World {
    message: String,
}

#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub struct EchoOptional {
    message: String,
    optional_message: Option<String>,
}

#[derive(Clone)]
pub struct McpService {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl McpService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Takes the input and combines it into a single output message")]
    pub async fn echo(&self, params: Parameters<EchoInput>) -> Result<Json<EchoOutput>, McpError> {
        let message = format!("{} {}", params.0.message, params.0.nested_item.message);
        Ok(Json(EchoOutput { message }))
    }

    #[tool(description = "Echoes a one-of value")]
    pub async fn echo_one_of(
        &self,
        params: Parameters<EchoOneOf>,
    ) -> Result<Json<EchoOutput>, McpError> {
        let message = match params.0.one_of {
            OneOf::Hello(hello) => format!("Hello Message: {}", hello.message),
            OneOf::World(world) => format!("World Message: {}", world.message),
        };
        Ok(Json(EchoOutput { message }))
    }

    #[tool(description = "Echoes an optional value")]
    pub async fn echo_optional(
        &self,
        params: Parameters<EchoOptional>,
    ) -> Result<Json<EchoOutput>, McpError> {
        let mut message = params.0.message.clone();
        if let Some(optional) = params.0.optional_message {
            message.push_str(&format!(" {}", optional));
        }
        Ok(Json(EchoOutput { message }))
    }
}

#[tool_handler]
impl ServerHandler for McpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some("A tool to echo messages".to_string()),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() {
    let config = StreamableHttpServerConfig {
        stateful_mode: false,
        ..Default::default()
    };
    let service = StreamableHttpService::new(
        || Ok(McpService::new()),
        NeverSessionManager {}.into(),
        config,
    );

    let app = Router::new().nest_service("/mcp", service);

    let address = "127.0.0.1:4000";
    let listener = TcpListener::bind(address).await.unwrap();
    println!("Server Listening on: http://{}", address);

    axum::serve(listener, app).await.unwrap();
}
