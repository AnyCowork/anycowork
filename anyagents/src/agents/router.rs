use crate::llm::LlmClient;
#[allow(unused_imports)]
use log::info;

/// Query complexity classification
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    /// Simple conversational query - just needs a direct answer
    Simple,
    /// Complex query that requires tools, file operations, or multi-step execution
    Complex,
}

pub struct Router {
    pub model: String,
    pub provider: String,
}

impl Router {
    pub fn new(model: String, provider: String) -> Self {
        Self { model, provider }
    }

    /// Classify a query as simple or complex
    pub async fn classify(&self, query: &str) -> QueryType {
        // Fast heuristics first - avoid LLM call for obvious cases
        let query_lower = query.to_lowercase();

        // Simple patterns - greetings, basic questions
        let simple_patterns = [
            "hello",
            "hi",
            "hey",
            "good morning",
            "good afternoon",
            "good evening",
            "how are you",
            "what's up",
            "thanks",
            "thank you",
            "bye",
            "goodbye",
            "what is",
            "what are",
            "who is",
            "who are",
            "why is",
            "why are",
            "explain",
            "tell me about",
            "describe",
            "define",
            "can you help",
            "help me understand",
        ];

        // Complex patterns - actions, file operations, code tasks
        let complex_patterns = [
            "create",
            "write",
            "make",
            "build",
            "generate",
            "implement",
            "edit",
            "modify",
            "change",
            "update",
            "fix",
            "refactor",
            "delete",
            "remove",
            "run",
            "execute",
            "install",
            "file",
            "folder",
            "directory",
            "code",
            "script",
            "search for",
            "find",
            "list files",
            "read file",
            "commit",
            "push",
            "pull",
            "deploy",
            "test",
            "debug",
            "compile",
            "lint",
        ];

        // Check for complex patterns first (they take priority)
        for pattern in complex_patterns.iter() {
            if query_lower.contains(pattern) {
                info!(
                    "Router: Query classified as COMPLEX (pattern: {})",
                    pattern
                );
                return QueryType::Complex;
            }
        }

        // Check for simple patterns
        for pattern in simple_patterns.iter() {
            if query_lower.starts_with(pattern) || query_lower.contains(pattern) {
                info!(
                    "Router: Query classified as SIMPLE (pattern: {})",
                    pattern
                );
                return QueryType::Simple;
            }
        }

        // For ambiguous cases, use LLM classification
        self.classify_with_llm(query).await
    }

    async fn classify_with_llm(&self, query: &str) -> QueryType {
        let preamble = r#"You are a query classifier. Classify the user's query into one of two categories:

SIMPLE - Queries that can be answered with just conversation/knowledge:
- Greetings and small talk
- General knowledge questions
- Explanations and definitions
- Opinions and advice
- Simple Q&A

COMPLEX - Queries that require tools, file operations, or multi-step execution:
- Creating, editing, or deleting files
- Running commands or scripts
- Searching codebases
- Building or deploying software
- Any task requiring system access

Respond with ONLY one word: "SIMPLE" or "COMPLEX""#;

        // Use a fast/cheap model for classification
        let fast_model = LlmClient::fast_model(&self.provider);
        let client = LlmClient::new(&self.provider, fast_model).with_preamble(preamble);

        match client.prompt(query).await {
            Ok(response) => {
                let response_upper = response.trim().to_uppercase();
                if response_upper.contains("SIMPLE") {
                    info!("Router: LLM classified as SIMPLE");
                    QueryType::Simple
                } else {
                    info!("Router: LLM classified as COMPLEX");
                    QueryType::Complex
                }
            }
            Err(_) => {
                // Default to complex if classification fails (safer)
                info!("Router: Classification failed, defaulting to COMPLEX");
                QueryType::Complex
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_router() -> Router {
        Router::new("test-model".to_string(), "test-provider".to_string())
    }

    #[tokio::test]
    async fn test_classify_simple_greeting() {
        let router = test_router();

        assert_eq!(router.classify("hello").await, QueryType::Simple);
        assert_eq!(router.classify("Hi there!").await, QueryType::Simple);
        assert_eq!(router.classify("Hey, how are you?").await, QueryType::Simple);
        assert_eq!(router.classify("Good morning!").await, QueryType::Simple);
    }

    #[tokio::test]
    async fn test_classify_simple_questions() {
        let router = test_router();

        assert_eq!(router.classify("What is Rust?").await, QueryType::Simple);
        assert_eq!(router.classify("Who is the CEO of OpenAI?").await, QueryType::Simple);
        assert_eq!(router.classify("Explain machine learning").await, QueryType::Simple);
        assert_eq!(router.classify("Tell me about quantum computing").await, QueryType::Simple);
        assert_eq!(router.classify("Define recursion").await, QueryType::Simple);
    }

    #[tokio::test]
    async fn test_classify_complex_file_operations() {
        let router = test_router();

        assert_eq!(router.classify("create a new file").await, QueryType::Complex);
        assert_eq!(router.classify("write a python script").await, QueryType::Complex);
        assert_eq!(router.classify("edit the config file").await, QueryType::Complex);
        assert_eq!(router.classify("delete the old logs").await, QueryType::Complex);
        assert_eq!(router.classify("read file contents").await, QueryType::Complex);
    }

    #[tokio::test]
    async fn test_classify_complex_code_tasks() {
        let router = test_router();

        assert_eq!(router.classify("implement a sorting algorithm").await, QueryType::Complex);
        assert_eq!(router.classify("fix the bug in main.rs").await, QueryType::Complex);
        assert_eq!(router.classify("refactor this function").await, QueryType::Complex);
        assert_eq!(router.classify("run the tests").await, QueryType::Complex);
        assert_eq!(router.classify("build the project").await, QueryType::Complex);
    }

    #[tokio::test]
    async fn test_classify_complex_git_operations() {
        let router = test_router();

        assert_eq!(router.classify("commit my changes").await, QueryType::Complex);
        assert_eq!(router.classify("push to origin").await, QueryType::Complex);
        assert_eq!(router.classify("pull the latest changes").await, QueryType::Complex);
    }

    #[tokio::test]
    async fn test_classify_complex_priority_over_simple() {
        let router = test_router();

        // Complex patterns should take priority even if simple patterns present
        assert_eq!(router.classify("Can you help me create a file?").await, QueryType::Complex);
        assert_eq!(router.classify("What is the best way to run this script?").await, QueryType::Complex);
        assert_eq!(router.classify("Tell me how to fix this bug").await, QueryType::Complex);
    }

    #[test]
    fn test_query_type_equality() {
        assert_eq!(QueryType::Simple, QueryType::Simple);
        assert_eq!(QueryType::Complex, QueryType::Complex);
        assert_ne!(QueryType::Simple, QueryType::Complex);
    }

    #[test]
    fn test_router_creation() {
        let router = Router::new("gpt-4".to_string(), "openai".to_string());
        assert_eq!(router.model, "gpt-4");
        assert_eq!(router.provider, "openai");
    }
}
