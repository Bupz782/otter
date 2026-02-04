/// Builds formatted prompts for the LLM
pub struct PromptBuilder {
    system_prompt: String,
}

impl PromptBuilder {
    pub fn new(system_prompt: impl Into<String>) -> Self {
        Self {
            system_prompt: system_prompt.into(),
        }
    }

    /// Build prompt in ChatML format (for Qwen3, etc.)
    /// Adds /no_think to disable thinking mode
    pub fn build(&self, user_input: &str) -> String {
        format!(
            "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{} /no_think<|im_end|>\n<|im_start|>assistant\n",
            self.system_prompt, user_input
        )
    }

    /// Build chat prompt in ChatML format
    pub fn build_chat(&self, history: &[ChatMessage]) -> String {
        let mut prompt = format!("<|im_start|>system\n{}<|im_end|>\n", self.system_prompt);

        for msg in history {
            let role = match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
            };
            prompt.push_str(&format!("<|im_start|>{}\n{}<|im_end|>\n", role, msg.content));
        }

        prompt.push_str("<|im_start|>assistant\n");
        prompt
    }

    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = prompt.into();
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new("You are a helpful assistant.")
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::System => "System",
            MessageRole::User => "User",
            MessageRole::Assistant => "Assistant",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder_chatml() {
        let builder = PromptBuilder::new("You are a DeFi parser.");
        let prompt = builder.build("swap 1 ETH for USDC");

        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("You are a DeFi parser."));
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("swap 1 ETH for USDC"));
        assert!(prompt.contains("<|im_start|>assistant"));
    }

    #[test]
    fn test_chat_messages_chatml() {
        let builder = PromptBuilder::new("You are a DeFi parser.");
        let history = vec![
            ChatMessage::user("Hello"),
            ChatMessage::assistant("Hi there!"),
            ChatMessage::user("swap 1 ETH"),
        ];

        let prompt = builder.build_chat(&history);
        assert!(prompt.contains("<|im_start|>user\nHello<|im_end|>"));
        assert!(prompt.contains("<|im_start|>assistant\nHi there!<|im_end|>"));
        assert!(prompt.contains("<|im_start|>user\nswap 1 ETH<|im_end|>"));
    }

    #[test]
    fn test_message_roles() {
        assert_eq!(MessageRole::System.as_str(), "System");
        assert_eq!(MessageRole::User.as_str(), "User");
        assert_eq!(MessageRole::Assistant.as_str(), "Assistant");
    }

    #[test]
    fn test_default_prompt_builder() {
        let builder = PromptBuilder::default();
        assert_eq!(builder.system_prompt(), "You are a helpful assistant.");
    }
}
