//! Hielements MCP Server
//!
//! Model Context Protocol server for exposing Hielements functionality to AI agents.
//! This server implements the MCP specification to provide:
//! - Resources: Read specifications, patterns, and library documentation
//! - Tools: Check, run, and generate Hielements specifications
//! - Prompts: Guidance templates for common agent tasks

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use clap::Parser;
use rust_mcp_sdk::error::SdkResult;
use rust_mcp_sdk::mcp_server::{server_runtime, McpServerOptions, ServerHandler, ServerRuntime};
use rust_mcp_sdk::schema::*;
use rust_mcp_sdk::{McpServer, StdioTransport, ToMcpServerHandler, TransportOptions};
use tracing::{info, Level};

mod resources;
mod tools;
mod prompts;

use resources::ResourceHandler;
use tools::ToolHandler;
use prompts::PromptHandler;

/// Command-line arguments for the MCP server
#[derive(Parser, Debug)]
#[command(name = "hielements-mcp")]
#[command(about = "MCP server for Hielements - expose architecture tools to AI agents")]
#[command(version)]
struct Args {
    /// Workspace directory containing Hielements specifications
    #[arg(short, long, default_value = ".")]
    workspace: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

/// Hielements MCP Handler
pub struct HielementsHandler {
    resources: ResourceHandler,
    tools: ToolHandler,
    prompts: PromptHandler,
}

impl HielementsHandler {
    pub fn new(workspace: PathBuf) -> Self {
        Self {
            resources: ResourceHandler::new(workspace.clone()),
            tools: ToolHandler::new(workspace),
            prompts: PromptHandler::new(),
        }
    }
}

#[async_trait]
impl ServerHandler for HielementsHandler {
    /// Handle list resources request
    async fn handle_list_resources_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListResourcesResult, RpcError> {
        let resources = self.resources.list_resources();
        Ok(ListResourcesResult {
            resources,
            meta: None,
            next_cursor: None,
        })
    }

    /// Handle read resource request
    async fn handle_read_resource_request(
        &self,
        params: ReadResourceRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ReadResourceResult, RpcError> {
        match self.resources.read_resource(&params.uri) {
            Ok(contents) => Ok(ReadResourceResult { contents, meta: None }),
            Err(e) => Err(RpcError::invalid_request().with_message(e)),
        }
    }

    /// Handle list tools request
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        let tools = self.tools.list_tools();
        Ok(ListToolsResult {
            tools,
            meta: None,
            next_cursor: None,
        })
    }

    /// Handle call tool request
    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        self.tools.call_tool(&params.name, params.arguments.unwrap_or_default())
    }

    /// Handle list prompts request
    async fn handle_list_prompts_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListPromptsResult, RpcError> {
        let prompts = self.prompts.list_prompts();
        Ok(ListPromptsResult {
            prompts,
            meta: None,
            next_cursor: None,
        })
    }

    /// Handle get prompt request
    async fn handle_get_prompt_request(
        &self,
        params: GetPromptRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<GetPromptResult, RpcError> {
        match self.prompts.get_prompt(&params.name, params.arguments) {
            Ok(result) => Ok(result),
            Err(e) => Err(RpcError::invalid_request().with_message(e)),
        }
    }
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    let args = Args::parse();

    // Initialize logging based on verbosity
    let level = if args.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .init();

    info!("Starting Hielements MCP server");
    info!("Workspace: {}", args.workspace);

    // Verify workspace exists
    let workspace_path = PathBuf::from(&args.workspace);
    if !workspace_path.exists() {
        eprintln!("Error: Workspace directory does not exist: {}", args.workspace);
        std::process::exit(1);
    }

    // Define server details and capabilities
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "hielements".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            title: Some("Hielements MCP Server".into()),
            description: Some("MCP server for Hielements - expose architecture tools to AI agents".into()),
            icons: vec![],
            website_url: Some("https://github.com/ercasta/hielements".into()),
        },
        capabilities: ServerCapabilities {
            resources: Some(ServerCapabilitiesResources {
                subscribe: None,
                list_changed: None,
            }),
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            prompts: Some(ServerCapabilitiesPrompts { list_changed: None }),
            ..Default::default()
        },
        protocol_version: ProtocolVersion::V2025_11_25.into(),
        instructions: Some("Hielements is a language for describing and enforcing software architecture. Use this server to validate specifications, run architectural checks, explore patterns, and get guidance on using the language.".into()),
        meta: None,
    };

    // Create transport and handler
    let transport = StdioTransport::new(TransportOptions::default())?;
    let handler = HielementsHandler::new(workspace_path);
    
    // Create and start the server
    let server: Arc<ServerRuntime> = server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler: handler.to_mcp_server_handler(),
        task_store: None,
        client_task_store: None,
    });
    
    info!("MCP server ready, waiting for connections...");
    
    if let Err(start_error) = server.start().await {
        eprintln!(
            "{}",
            start_error
                .rpc_error_message()
                .unwrap_or(&start_error.to_string())
        );
    }
    
    Ok(())
}
