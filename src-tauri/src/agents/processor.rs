

pub enum StreamChunk {
    Text(String),
    Thinking(String),
}

pub struct StreamProcessor {
    in_thinking_block: bool,
    buffer: String,
}

impl StreamProcessor {
    pub fn new() -> Self {
        Self {
            in_thinking_block: false,
            buffer: String::new(),
        }
    }

    pub fn process(&mut self, token: &str) -> Vec<StreamChunk> {
        let mut chunks = Vec::new();
        let current_token = token.to_string();

        // Simple state machine for <think> tags
        // This is a naive implementation; a robust one would handle split tags across tokens.
        // For MVP, checking if token contains tag is a starting point, but we need buffering.
        
        self.buffer.push_str(&current_token);
        
        // Process buffer
        loop {
            if self.in_thinking_block {
                if let Some(end_idx) = self.buffer.find("</think>") {
                    // Found end of thinking
                    let thinking_content = self.buffer[..end_idx].to_string();
                    if !thinking_content.is_empty() {
                         chunks.push(StreamChunk::Thinking(thinking_content));
                    }
                    self.buffer = self.buffer[end_idx + 8..].to_string();
                    self.in_thinking_block = false;
                } else {
                    // Still in thinking, emit all as thinking (except potential partial tag at end?)
                    // For safety, just emit what we have and clear buffer? 
                    // No, we might lose context if we don't wait for closing tag?
                    // StreamTextResult usually streams tokens.
                    // If we stick to "emit thinking as it comes", we can just emit.
                    if !self.buffer.is_empty() {
                         chunks.push(StreamChunk::Thinking(self.buffer.clone()));
                         self.buffer.clear();
                    }
                    break;
                }
            } else {
                if let Some(start_idx) = self.buffer.find("<think>") {
                    // Found start of thinking
                    let text_content = self.buffer[..start_idx].to_string();
                    if !text_content.is_empty() {
                        chunks.push(StreamChunk::Text(text_content));
                    }
                    self.buffer = self.buffer[start_idx + 7..].to_string();
                    self.in_thinking_block = true;
                } else {
                    // No thinking tag, all text
                    // But wait, what if "<th" is at the end?
                    // We need to keep potential partial tag.
                    if self.buffer.ends_with("<") || self.buffer.ends_with("<t") || self.buffer.ends_with("<th") || self.buffer.ends_with("<thi") || self.buffer.ends_with("<thin") || self.buffer.ends_with("<think") {
                         // Keep in buffer wait for next token
                         break;
                    }
                    
                    if !self.buffer.is_empty() {
                        chunks.push(StreamChunk::Text(self.buffer.clone()));
                        self.buffer.clear();
                    }
                    break;
                }
            }
        }
        
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_processor_text_only() {
        let mut processor = StreamProcessor::new();
        let chunks = processor.process("Hello world");
        assert_eq!(chunks.len(), 1);
        if let StreamChunk::Text(t) = &chunks[0] {
            assert_eq!(t, "Hello world");
        } else { panic!("Expected Text"); }
    }

    #[test]
    fn test_stream_processor_thinking_block() {
        let mut processor = StreamProcessor::new();
        let chunks1 = processor.process("<think>This is");
        assert_eq!(chunks1.len(), 1); 
        if let StreamChunk::Thinking(t) = &chunks1[0] {
            assert_eq!(t, "This is");
        } else { panic!("Expected Thinking"); }

        let chunks2 = processor.process(" reasoning</think>");
        assert_eq!(chunks2.len(), 1);
        if let StreamChunk::Thinking(t) = &chunks2[0] {
            assert_eq!(t, " reasoning");
        } else { panic!("Expected Thinking"); }
    }

    #[test]
    fn test_stream_processor_mixed() {
        let mut processor = StreamProcessor::new();
        let chunks = processor.process("Start <think>reason</think> End");
        // "Start " -> Text
        // "reason" -> Thinking
        // " End" -> Text
        
        assert_eq!(chunks.len(), 3);
        match &chunks[0] { StreamChunk::Text(t) => assert_eq!(t, "Start "), _ => panic!("1") }
        match &chunks[1] { StreamChunk::Thinking(t) => assert_eq!(t, "reason"), _ => panic!("2") }
        match &chunks[2] { StreamChunk::Text(t) => assert_eq!(t, " End"), _ => panic!("3") }
    }
}
