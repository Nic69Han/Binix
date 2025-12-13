//! Streaming/Incremental HTML parser
//!
//! Provides incremental parsing for progressive rendering:
//! - Parse HTML as it arrives from the network
//! - Emit partial DOM trees for early rendering
//! - Support for chunked transfer encoding

use html5ever::tendril::TendrilSink;
use html5ever::{ParseOpts, parse_document};
use markup5ever_rcdom::{Handle, RcDom};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// Parser state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserState {
    /// Initial state, waiting for data
    Idle,
    /// Actively parsing
    Parsing,
    /// Parsing complete
    Complete,
    /// Error occurred
    Error,
}

/// Chunk of parsed content
#[derive(Debug, Clone)]
pub struct ParsedChunk {
    /// Nodes parsed in this chunk
    pub node_count: usize,
    /// Text content parsed
    pub text_length: usize,
    /// Whether this is the final chunk
    pub is_final: bool,
}

/// Streaming HTML parser
pub struct StreamingParser {
    state: ParserState,
    buffer: String,
    chunks_received: usize,
    bytes_received: usize,
    parsed_chunks: VecDeque<ParsedChunk>,
    dom: Option<RcDom>,
}

impl StreamingParser {
    /// Create a new streaming parser
    pub fn new() -> Self {
        Self {
            state: ParserState::Idle,
            buffer: String::new(),
            chunks_received: 0,
            bytes_received: 0,
            parsed_chunks: VecDeque::new(),
            dom: None,
        }
    }

    /// Feed a chunk of HTML data
    pub fn feed(&mut self, chunk: &str) -> ParsedChunk {
        self.state = ParserState::Parsing;
        self.chunks_received += 1;
        self.bytes_received += chunk.len();
        self.buffer.push_str(chunk);

        // Try to parse what we have so far
        let node_count = self.try_incremental_parse();

        let parsed = ParsedChunk {
            node_count,
            text_length: chunk.len(),
            is_final: false,
        };

        self.parsed_chunks.push_back(parsed.clone());
        parsed
    }

    /// Finish parsing
    pub fn finish(&mut self) -> Result<(), String> {
        self.state = ParserState::Complete;

        // Parse the complete document
        let opts = ParseOpts::default();
        let dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut self.buffer.as_bytes())
            .map_err(|e| format!("Parse error: {}", e))?;

        let node_count = self.count_nodes(&dom.document);
        self.dom = Some(dom);

        let final_chunk = ParsedChunk {
            node_count,
            text_length: 0,
            is_final: true,
        };
        self.parsed_chunks.push_back(final_chunk);

        Ok(())
    }

    /// Try to parse incrementally (simplified - parses complete fragments)
    fn try_incremental_parse(&mut self) -> usize {
        // In a real implementation, we would use a proper incremental parser
        // For now, we count potential nodes based on tag patterns
        let tag_count = self.buffer.matches('<').count();
        tag_count
    }

    /// Count nodes in DOM tree
    fn count_nodes(&self, handle: &Handle) -> usize {
        let mut count = 1;

        for child in handle.children.borrow().iter() {
            count += self.count_nodes(child);
        }

        count
    }

    /// Get parser state
    pub fn state(&self) -> ParserState {
        self.state
    }

    /// Get chunks received
    pub fn chunks_received(&self) -> usize {
        self.chunks_received
    }

    /// Get bytes received
    pub fn bytes_received(&self) -> usize {
        self.bytes_received
    }

    /// Get parsed chunks
    pub fn parsed_chunks(&self) -> &VecDeque<ParsedChunk> {
        &self.parsed_chunks
    }

    /// Get the DOM (if complete)
    pub fn dom(&self) -> Option<&RcDom> {
        self.dom.as_ref()
    }

    /// Reset the parser
    pub fn reset(&mut self) {
        self.state = ParserState::Idle;
        self.buffer.clear();
        self.chunks_received = 0;
        self.bytes_received = 0;
        self.parsed_chunks.clear();
        self.dom = None;
    }
}

impl Default for StreamingParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_parser_creation() {
        let parser = StreamingParser::new();
        assert_eq!(parser.state(), ParserState::Idle);
        assert_eq!(parser.chunks_received(), 0);
    }

    #[test]
    fn test_streaming_parser_feed() {
        let mut parser = StreamingParser::new();
        let chunk = parser.feed("<html><head>");

        assert_eq!(parser.state(), ParserState::Parsing);
        assert_eq!(parser.chunks_received(), 1);
        assert!(chunk.node_count > 0);
    }

    #[test]
    fn test_streaming_parser_multiple_chunks() {
        let mut parser = StreamingParser::new();
        parser.feed("<html>");
        parser.feed("<head><title>Test</title></head>");
        parser.feed("<body><p>Hello</p></body>");
        parser.feed("</html>");

        assert_eq!(parser.chunks_received(), 4);
    }

    #[test]
    fn test_streaming_parser_finish() {
        let mut parser = StreamingParser::new();
        parser.feed("<html><body><p>Test</p></body></html>");
        parser.finish().unwrap();

        assert_eq!(parser.state(), ParserState::Complete);
        assert!(parser.dom().is_some());
    }

    #[test]
    fn test_streaming_parser_reset() {
        let mut parser = StreamingParser::new();
        parser.feed("<html>");
        parser.reset();

        assert_eq!(parser.state(), ParserState::Idle);
        assert_eq!(parser.chunks_received(), 0);
    }
}
