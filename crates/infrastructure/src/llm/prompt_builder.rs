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

    pub fn build(&self, user_input: &str) -> String {
        format!(
            "{}\n\nUser input: {}\n\nAssistant:",
            self.system_prompt, user_input
        )
    }

    pub fn build_chat(&self, history: &[ChatMessage]) -> String {
        let mut prompt = self.system_prompt.clone();

        for msg in history {
            prompt.push_str(&format!("\n\n{}: {}", msg.role.as_str(), msg.content));
        }

        prompt.push_str("\n\nAssistant:");
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
    fn test_prompt_builder() {
        let builder = PromptBuilder::new("You are a DeFi parser.");
        let prompt = builder.build("swap 1 ETH for USDC");

        assert!(prompt.contains("You are a DeFi parser."));
        assert!(prompt.contains("User input: swap 1 ETH for USDC"));
        assert!(prompt.contains("Assistant:"));
    }

    #[test]
    fn test_chat_messages() {
        let builder = PromptBuilder::new("You are a DeFi parser.");
        let history = vec![
            ChatMessage::user("Hello"),
            ChatMessage::assistant("Hi there!"),
            ChatMessage::user("swap 1 ETH"),
        ];

        let prompt = builder.build_chat(&history);
        assert!(prompt.contains("User: Hello"));
        assert!(prompt.contains("Assistant: Hi there!"));
        assert!(prompt.contains("User: swap 1 ETH"));
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
