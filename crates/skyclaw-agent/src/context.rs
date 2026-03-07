//! Context builder — assembles a CompletionRequest from session history,
//! memory search results, system prompt, and tool definitions.

use std::sync::Arc;

use skyclaw_core::Memory;
use skyclaw_core::SearchOpts;
use skyclaw_core::Tool;
use skyclaw_core::types::message::{
    ChatMessage, CompletionRequest, MessageContent, Role, ToolDefinition,
};
use skyclaw_core::types::session::SessionContext;

/// Estimate token count from a string (rough: 1 token ≈ 4 chars).
fn estimate_tokens(s: &str) -> usize {
    s.len() / 4
}

/// Estimate token count for a ChatMessage.
fn estimate_message_tokens(msg: &ChatMessage) -> usize {
    match &msg.content {
        MessageContent::Text(t) => estimate_tokens(t),
        MessageContent::Parts(parts) => {
            parts.iter().map(|p| match p {
                skyclaw_core::types::message::ContentPart::Text { text } => estimate_tokens(text),
                skyclaw_core::types::message::ContentPart::ToolUse { input, .. } => estimate_tokens(&input.to_string()),
                skyclaw_core::types::message::ContentPart::ToolResult { content, .. } => estimate_tokens(content),
            }).sum()
        }
    }
}

/// Build a CompletionRequest from all available context.
pub async fn build_context(
    session: &SessionContext,
    memory: &dyn Memory,
    tools: &[Arc<dyn Tool>],
    model: &str,
    system_prompt: Option<&str>,
    max_turns: usize,
    max_context_tokens: usize,
) -> CompletionRequest {
    let mut messages: Vec<ChatMessage> = Vec::new();

    // 1. Retrieve relevant memory entries for context augmentation
    let query = session
        .history
        .last()
        .and_then(|m| match &m.content {
            MessageContent::Text(t) => Some(t.clone()),
            MessageContent::Parts(parts) => parts.iter().find_map(|p| match p {
                skyclaw_core::types::message::ContentPart::Text { text } => Some(text.clone()),
                _ => None,
            }),
        })
        .unwrap_or_default();

    if !query.is_empty() {
        let opts = SearchOpts {
            limit: 5,
            session_filter: Some(session.session_id.clone()),
            ..Default::default()
        };

        if let Ok(entries) = memory.search(&query, opts).await {
            if !entries.is_empty() {
                let memory_text: String = entries
                    .iter()
                    .map(|e| format!("[{}] {}", e.timestamp.format("%Y-%m-%d %H:%M"), e.content))
                    .collect::<Vec<_>>()
                    .join("\n");

                messages.push(ChatMessage {
                    role: Role::System,
                    content: MessageContent::Text(format!(
                        "Relevant context from memory:\n{}",
                        memory_text
                    )),
                });
            }
        }
    }

    // 2. Trim session history to max_turns pairs, keeping the most recent
    let history = &session.history;
    let trimmed: Vec<ChatMessage> = if max_turns > 0 && history.len() > max_turns * 2 {
        // Keep the last N*2 messages (N pairs of user+assistant/tool)
        history[history.len() - max_turns * 2..].to_vec()
    } else {
        history.clone()
    };

    // 3. Apply token budget — drop oldest messages until under limit
    let system_tokens = messages.iter().map(|m| estimate_message_tokens(m)).sum::<usize>();
    let tool_def_tokens: usize = tools.iter().map(|t| {
        estimate_tokens(t.name()) + estimate_tokens(t.description()) + estimate_tokens(&t.parameters_schema().to_string())
    }).sum();
    let base_tokens = system_tokens + tool_def_tokens + 500; // 500 for overhead

    let mut kept: Vec<ChatMessage> = Vec::new();
    let mut total_tokens = base_tokens;
    // Walk from newest to oldest, accumulate until budget exceeded
    for msg in trimmed.iter().rev() {
        let msg_tokens = estimate_message_tokens(msg);
        if total_tokens + msg_tokens > max_context_tokens {
            break;
        }
        total_tokens += msg_tokens;
        kept.push(msg.clone());
    }
    kept.reverse();
    messages.extend(kept);

    // 3. Build tool definitions
    let tool_defs: Vec<ToolDefinition> = tools
        .iter()
        .map(|t| ToolDefinition {
            name: t.name().to_string(),
            description: t.description().to_string(),
            parameters: t.parameters_schema(),
        })
        .collect();

    // 4. Assemble the system prompt
    let system = system_prompt.map(|s| s.to_string()).or_else(|| {
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        Some(format!(
            "You are SkyClaw, a cloud-native AI agent runtime. You control a computer through messaging apps.\n\
             \n\
             You have access to these tools: {}\n\
             \n\
             Workspace: All file operations use the workspace directory at {}.\n\
             Files sent by the user are automatically saved here.\n\
             \n\
             File protocol:\n\
             - Received files are saved to the workspace automatically — use file_read to read them\n\
             - To send a file to the user, use send_file with just the path (chat_id is automatic)\n\
             - Use file_write to create files in the workspace, then send_file to deliver them\n\
             - Paths are relative to the workspace directory\n\
             \n\
             Guidelines:\n\
             - Use the shell tool to run commands, install packages, manage services, check system status\n\
             - Use file tools to read, write, and list files in the workspace\n\
             - Use web_fetch to look up documentation, check APIs, or research information\n\
             - Be concise in responses — the user is on a messaging app\n\
             - When a task requires multiple steps, execute them sequentially using tools\n\
             - If a command fails, read the error and try to fix it\n\
             - Never expose secrets, API keys, or sensitive data in responses",
            tool_names.join(", "),
            session.workspace_path.display()
        ))
    });

    CompletionRequest {
        model: model.to_string(),
        messages,
        tools: tool_defs,
        max_tokens: Some(4096),
        temperature: Some(0.7),
        system,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skyclaw_test_utils::{MockMemory, MockTool, make_session};

    #[tokio::test]
    async fn context_includes_system_prompt() {
        let memory = MockMemory::new();
        let tools: Vec<Arc<dyn Tool>> = vec![];
        let session = make_session();

        let req = build_context(&session, &memory, &tools, "test-model", Some("Custom prompt"), 6, 30_000).await;
        assert_eq!(req.system.as_deref(), Some("Custom prompt"));
        assert_eq!(req.model, "test-model");
    }

    #[tokio::test]
    async fn context_default_system_prompt() {
        let memory = MockMemory::new();
        let tools: Vec<Arc<dyn Tool>> = vec![];
        let session = make_session();

        let req = build_context(&session, &memory, &tools, "test-model", None, 6, 30_000).await;
        assert!(req.system.is_some());
        assert!(req.system.unwrap().contains("SkyClaw"));
    }

    #[tokio::test]
    async fn context_includes_tool_definitions() {
        let memory = MockMemory::new();
        let tools: Vec<Arc<dyn Tool>> = vec![
            Arc::new(MockTool::new("shell")),
            Arc::new(MockTool::new("browser")),
        ];
        let session = make_session();

        let req = build_context(&session, &memory, &tools, "model", None, 6, 30_000).await;
        assert_eq!(req.tools.len(), 2);
        assert_eq!(req.tools[0].name, "shell");
        assert_eq!(req.tools[1].name, "browser");
    }

    #[tokio::test]
    async fn context_includes_conversation_history() {
        let memory = MockMemory::new();
        let tools: Vec<Arc<dyn Tool>> = vec![];
        let mut session = make_session();
        session.history.push(ChatMessage {
            role: Role::User,
            content: MessageContent::Text("Hello".to_string()),
        });
        session.history.push(ChatMessage {
            role: Role::Assistant,
            content: MessageContent::Text("Hi there".to_string()),
        });

        let req = build_context(&session, &memory, &tools, "model", None, 6, 30_000).await;
        // Messages should include the history
        assert!(req.messages.len() >= 2);
    }
}
