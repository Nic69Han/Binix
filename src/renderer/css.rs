//! CSS3 parser and stylesheet representation

use crate::utils::{error::RenderError, Result};
use std::collections::HashMap;

/// CSS value types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Keyword (e.g., "auto", "none")
    Keyword(String),
    /// Length with unit (e.g., 10px)
    Length(f32, Unit),
    /// Color value
    Color(Color),
    /// Percentage
    Percentage(f32),
}

/// CSS length units
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px,
    Em,
    Rem,
    Percent,
    Vh,
    Vw,
}

/// CSS color
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// CSS selector
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

impl Selector {
    /// Calculate specificity (a, b, c)
    pub fn specificity(&self) -> (u32, u32, u32) {
        let a = if self.id.is_some() { 1 } else { 0 };
        let b = self.classes.len() as u32;
        let c = if self.tag_name.is_some() { 1 } else { 0 };
        (a, b, c)
    }
}

/// CSS declaration (property: value)
#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: Value,
}

/// CSS rule (selector + declarations)
#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

/// CSS stylesheet
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

/// CSS parser
pub struct CssParser {}

impl CssParser {
    pub fn new() -> Self {
        Self {}
    }

    /// Parse CSS content into a stylesheet
    pub fn parse(&self, content: &str) -> Result<Stylesheet> {
        // TODO: Implement full CSS3 parsing
        // For now, return an empty stylesheet
        Ok(Stylesheet::default())
    }
}

impl Default for CssParser {
    fn default() -> Self {
        Self::new()
    }
}

