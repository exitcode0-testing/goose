use crate::state::AppState;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

// Local wrapper type for SamplingMessage to avoid OpenAPI generation issues with external types
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SamplingMessage {
    /// The role of the message sender (User or Assistant)
    pub role: String,
    /// The actual content of the message
    pub content: String,
}

impl From<rmcp::model::SamplingMessage> for SamplingMessage {
    fn from(msg: rmcp::model::SamplingMessage) -> Self {
        // Convert Content to string representation
        let content_str = match msg.content.as_text() {
            Some(text) => text.text.to_string(),
            None => serde_json::to_string(&msg.content).unwrap_or_else(|_| "".to_string()),
        };
        
        Self {
            role: format!("{:?}", msg.role).to_lowercase(),
            content: content_str,
        }
    }
}

impl From<SamplingMessage> for rmcp::model::SamplingMessage {
    fn from(msg: SamplingMessage) -> Self {
        use rmcp::model::{Content, Role};
        
        let role = match msg.role.as_str() {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            _ => Role::User,
        };
        
        rmcp::model::SamplingMessage {
            role,
            content: Content::text(msg.content),
        }
    }
}

// Local wrapper for SamplingConfirmationRequest
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SamplingConfirmationRequest {
    pub id: String,
    pub extension_name: String,
    pub messages: Vec<SamplingMessage>,
    pub system_prompt: Option<String>,
    pub prompt: Option<String>,
}

impl From<goose::conversation::message::SamplingConfirmationRequest> for SamplingConfirmationRequest {
    fn from(req: goose::conversation::message::SamplingConfirmationRequest) -> Self {
        Self {
            id: req.id,
            extension_name: req.extension_name,
            messages: req.messages.into_iter().map(SamplingMessage::from).collect(),
            system_prompt: req.system_prompt,
            prompt: req.prompt,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SamplingApprovalRequest {
    /// The session ID for the current agent session
    pub session_id: String,
    /// The sampling confirmation request ID
    pub id: String,
    /// The user's action: approve, deny, or edit
    pub action: String,
    /// If action is "edit", this contains the edited messages
    pub edited_messages: Option<Vec<SamplingMessage>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SamplingResponse {
    /// Status of the response
    pub status: String,
    /// Optional message
    pub message: Option<String>,
}

#[utoipa::path(
    post,
    path = "/sampling/approve",
    request_body = SamplingApprovalRequest,
    responses(
        (status = 200, description = "Sampling approval processed", body = SamplingResponse),
        (status = 401, description = "Unauthorized - invalid secret key"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn handle_sampling_approval(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SamplingApprovalRequest>,
) -> Result<Json<SamplingResponse>, StatusCode> {
    // Get the agent for this session
    let _agent = state.get_agent_for_route(request.session_id.clone()).await?;
    
    // Handle the sampling confirmation based on the action
    match request.action.as_str() {
        "approve" => {
            // TODO: Send approval to the agent's sampling handler
            // For now, we'll just acknowledge the request
            Ok(Json(SamplingResponse {
                status: "approved".to_string(),
                message: Some("Sampling request approved".to_string()),
            }))
        }
        "deny" => {
            // TODO: Send denial to the agent's sampling handler
            Ok(Json(SamplingResponse {
                status: "denied".to_string(),
                message: Some("Sampling request denied".to_string()),
            }))
        }
        "edit" => {
            // TODO: Handle edited messages
            Ok(Json(SamplingResponse {
                status: "edited".to_string(),
                message: Some("Sampling request edited and approved".to_string()),
            }))
        }
        _ => {
            Ok(Json(SamplingResponse {
                status: "error".to_string(),
                message: Some("Invalid action".to_string()),
            }))
        }
    }
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/sampling/approve", post(handle_sampling_approval))
        .with_state(state)
}
