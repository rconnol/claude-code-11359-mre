use axum::Router;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::InitializeRequestParam;
use rmcp::model::InitializeResult;
use rmcp::model::ServerCapabilities;
use rmcp::model::ServerInfo;
use rmcp::model::ToolsCapability;
use rmcp::service::RequestContext;
use rmcp::tool;
use rmcp::tool_router;
use rmcp::transport::sse_server::SseServerConfig;
use rmcp::transport::SseServer;
use rmcp::ErrorData as McpError;
use rmcp::Json;
use rmcp::RoleServer;
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
}

impl ServerHandler for McpService {
    async fn initialize(
        &self,
        request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        // Set peer info first (standard behavior)
        if context.peer.peer_info().is_none() {
            context.peer.set_peer_info(request);
        }

        // Create capabilities that properly advertise our tools
        let capabilities = ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            prompts: None,
            resources: None,
            logging: None,
            experimental: None,
            completions: None,
        };

        let server_info = self.get_info();
        let implementation = rmcp::model::Implementation {
            name: "echo".to_string(),
            title: None,
            version: "0.1.0".to_string(),
            icons: None,
            website_url: None,
        };
        Ok(InitializeResult {
            protocol_version: Default::default(),
            capabilities,
            server_info: implementation,
            instructions: server_info.instructions.clone(),
        })
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A tool to echo messages".to_string()),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() {
    let (sse, mcp_router) = SseServer::new(SseServerConfig {
        bind: "127.0.0.1:0".parse().unwrap(),
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: tokio_util::sync::CancellationToken::new(),
        sse_keep_alive: None,
    });

    let address = "127.0.0.1:4000";
    let _ct = sse.with_service_directly(move || {
        use rmcp::handler::server::router::Router as McpRouter;
        let service = McpService::new();
        McpRouter::new(service.clone()).with_tools(service.tool_router)
    });

    let app = Router::new().nest("/mcp", mcp_router);

    let listener = TcpListener::bind(address).await.unwrap();
    println!("Server Listening on: http://{}", address);

    axum::serve(listener, app).await.unwrap();
}
