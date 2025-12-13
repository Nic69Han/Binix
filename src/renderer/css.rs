//! CSS3 parser and stylesheet representation using cssparser

use crate::utils::Result;
use cssparser::{Parser, ParserInput, Token, ParseError, BasicParseErrorKind};

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
    /// Number without unit
    Number(f32),
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
    Pt,
    Cm,
    Mm,
    In,
}

impl Unit {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "px" => Some(Unit::Px),
            "em" => Some(Unit::Em),
            "rem" => Some(Unit::Rem),
            "%" => Some(Unit::Percent),
            "vh" => Some(Unit::Vh),
            "vw" => Some(Unit::Vw),
            "pt" => Some(Unit::Pt),
            "cm" => Some(Unit::Cm),
            "mm" => Some(Unit::Mm),
            "in" => Some(Unit::In),
            _ => None,
        }
    }
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

    /// Parse a hex color string
    fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Color::rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Parse named colors
    fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "black" => Some(Color::rgb(0, 0, 0)),
            "white" => Some(Color::rgb(255, 255, 255)),
            "red" => Some(Color::rgb(255, 0, 0)),
            "green" => Some(Color::rgb(0, 128, 0)),
            "blue" => Some(Color::rgb(0, 0, 255)),
            "yellow" => Some(Color::rgb(255, 255, 0)),
            "cyan" => Some(Color::rgb(0, 255, 255)),
            "magenta" => Some(Color::rgb(255, 0, 255)),
            "gray" | "grey" => Some(Color::rgb(128, 128, 128)),
            "orange" => Some(Color::rgb(255, 165, 0)),
            "purple" => Some(Color::rgb(128, 0, 128)),
            "pink" => Some(Color::rgb(255, 192, 203)),
            "transparent" => Some(Color::rgba(0, 0, 0, 0)),
            _ => None,
        }
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

/// CSS parser using cssparser crate
pub struct CssParser {}

impl CssParser {
    pub fn new() -> Self {
        Self {}
    }

    /// Parse CSS content into a stylesheet
    pub fn parse(&self, content: &str) -> Result<Stylesheet> {
        let mut input = ParserInput::new(content);
        let mut parser = Parser::new(&mut input);
        let mut rules = Vec::new();

        while !parser.is_exhausted() {
            // Skip whitespace
            let _ = parser.try_parse::<_, _, ParseError<()>>(|p| {
                p.skip_whitespace();
                Ok(())
            });

            if parser.is_exhausted() {
                break;
            }

            // Try to parse a rule
            if let Ok(rule) = self.parse_rule(&mut parser) {
                rules.push(rule);
            } else {
                // Skip to next rule on error
                let _ = self.skip_to_next_rule(&mut parser);
            }
        }

        Ok(Stylesheet { rules })
    }

    /// Parse a single CSS rule
    fn parse_rule<'i>(&self, parser: &mut Parser<'i, '_>) -> std::result::Result<Rule, ParseError<'i, ()>> {
        // Collect selector tokens until we hit a curly brace
        let mut selector_str = String::new();

        loop {
            parser.skip_whitespace();
            let state = parser.state();
            match parser.next() {
                Ok(Token::CurlyBracketBlock) => {
                    // Parse declarations inside the block
                    let declarations = parser.parse_nested_block(|p| self.parse_declarations(p))?;
                    let selectors = self.parse_selector_string(&selector_str);
                    return Ok(Rule { selectors, declarations });
                }
                Ok(token) => {
                    // Append token to selector string
                    selector_str.push_str(&self.token_to_string(token));
                }
                Err(_) => {
                    parser.reset(&state);
                    return Err(parser.new_error(BasicParseErrorKind::EndOfInput));
                }
            }
        }
    }

    /// Convert a token to its string representation
    fn token_to_string(&self, token: &Token) -> String {
        match token {
            Token::Ident(s) => s.to_string(),
            Token::IDHash(s) => format!("#{}", s),
            Token::Hash(s) => format!("#{}", s),
            Token::Delim(c) => c.to_string(),
            Token::Comma => ",".to_string(),
            Token::WhiteSpace(_) => " ".to_string(),
            Token::Colon => ":".to_string(),
            _ => String::new(),
        }
    }

    /// Parse selector string into Selector structs
    fn parse_selector_string(&self, selector_str: &str) -> Vec<Selector> {
        selector_str
            .split(',')
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    return None;
                }
                Some(self.parse_single_selector(s))
            })
            .collect()
    }

    /// Parse a single selector from string
    fn parse_single_selector(&self, s: &str) -> Selector {
        let mut tag_name = None;
        let mut id = None;
        let mut classes = Vec::new();

        let mut current = String::new();
        let mut mode = 'T'; // T=tag, I=id, C=class

        for c in s.chars() {
            match c {
                '#' => {
                    if !current.is_empty() {
                        match mode {
                            'T' => tag_name = Some(current.clone()),
                            'I' => id = Some(current.clone()),
                            'C' => classes.push(current.clone()),
                            _ => {}
                        }
                        current.clear();
                    }
                    mode = 'I';
                }
                '.' => {
                    if !current.is_empty() {
                        match mode {
                            'T' => tag_name = Some(current.clone()),
                            'I' => id = Some(current.clone()),
                            'C' => classes.push(current.clone()),
                            _ => {}
                        }
                        current.clear();
                    }
                    mode = 'C';
                }
                ' ' | '\t' | '\n' | '\r' => {
                    // Skip whitespace for now (simple selector only)
                    if !current.is_empty() {
                        match mode {
                            'T' => tag_name = Some(current.clone()),
                            'I' => id = Some(current.clone()),
                            'C' => classes.push(current.clone()),
                            _ => {}
                        }
                        current.clear();
                        mode = 'T';
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        // Handle remaining content
        if !current.is_empty() {
            match mode {
                'T' => tag_name = Some(current),
                'I' => id = Some(current),
                'C' => classes.push(current),
                _ => {}
            }
        }

        Selector { tag_name, id, classes }
    }

    /// Parse declarations inside a rule block
    fn parse_declarations<'i>(&self, parser: &mut Parser<'i, '_>) -> std::result::Result<Vec<Declaration>, ParseError<'i, ()>> {
        let mut declarations = Vec::new();

        loop {
            parser.skip_whitespace();
            if parser.is_exhausted() {
                break;
            }

            // Try to parse a declaration
            let result: std::result::Result<Declaration, ParseError<'i, ()>> = parser.try_parse(|p| {
                let property = p.expect_ident()?.to_string();
                p.expect_colon()?;
                p.skip_whitespace();
                let value = self.parse_value(p)?;

                // Optional semicolon
                let _ = p.try_parse::<_, _, ParseError<()>>(|p2| {
                    p2.expect_semicolon()?;
                    Ok(())
                });

                Ok(Declaration { property, value })
            });

            match result {
                Ok(decl) => declarations.push(decl),
                Err(_) => {
                    // Skip to next declaration
                    let _ = self.skip_to_semicolon(parser);
                }
            }
        }

        Ok(declarations)
    }

    /// Parse a CSS value
    fn parse_value<'i>(&self, parser: &mut Parser<'i, '_>) -> std::result::Result<Value, ParseError<'i, ()>> {
        parser.skip_whitespace();

        let token = parser.next()?.clone();
        match token {
            Token::Number { value, .. } => Ok(Value::Number(value)),
            Token::Percentage { unit_value, .. } => Ok(Value::Percentage(unit_value * 100.0)),
            Token::Dimension { value, unit, .. } => {
                if let Some(u) = Unit::from_str(&unit) {
                    Ok(Value::Length(value, u))
                } else {
                    Ok(Value::Keyword(format!("{}{}", value, unit)))
                }
            }
            Token::Ident(name) => {
                let name_str = name.to_string();
                // Check if it's a color name
                if let Some(color) = Color::from_name(&name_str) {
                    Ok(Value::Color(color))
                } else {
                    Ok(Value::Keyword(name_str))
                }
            }
            Token::IDHash(hash) | Token::Hash(hash) => {
                // Hex color
                if let Some(color) = Color::from_hex(&hash) {
                    Ok(Value::Color(color))
                } else {
                    Ok(Value::Keyword(format!("#{}", hash)))
                }
            }
            Token::Function(name) => {
                let name_str = name.to_string();
                // Parse function arguments
                if name_str == "rgb" || name_str == "rgba" {
                    parser.parse_nested_block(|p| self.parse_rgb_function(p, name_str == "rgba"))
                } else {
                    // Skip function content
                    parser.parse_nested_block(|p| {
                        while p.next().is_ok() {}
                        Ok(Value::Keyword(name_str))
                    })
                }
            }
            _ => Err(parser.new_error(BasicParseErrorKind::UnexpectedToken(token))),
        }
    }

    /// Parse rgb() or rgba() function
    fn parse_rgb_function<'i>(&self, parser: &mut Parser<'i, '_>, has_alpha: bool) -> std::result::Result<Value, ParseError<'i, ()>> {
        let r = self.parse_color_component(parser)?;
        let _ = parser.try_parse::<_, _, ParseError<()>>(|p| { p.expect_comma()?; Ok(()) });
        let g = self.parse_color_component(parser)?;
        let _ = parser.try_parse::<_, _, ParseError<()>>(|p| { p.expect_comma()?; Ok(()) });
        let b = self.parse_color_component(parser)?;

        let a = if has_alpha {
            let _ = parser.try_parse::<_, _, ParseError<()>>(|p| { p.expect_comma()?; Ok(()) });
            self.parse_alpha_component(parser)?
        } else {
            255
        };

        Ok(Value::Color(Color::rgba(r, g, b, a)))
    }

    /// Parse a color component (0-255 or percentage)
    fn parse_color_component<'i>(&self, parser: &mut Parser<'i, '_>) -> std::result::Result<u8, ParseError<'i, ()>> {
        parser.skip_whitespace();
        let token = parser.next()?.clone();
        match token {
            Token::Number { value, .. } => Ok((value.clamp(0.0, 255.0)) as u8),
            Token::Percentage { unit_value, .. } => Ok((unit_value * 255.0).clamp(0.0, 255.0) as u8),
            _ => Err(parser.new_error(BasicParseErrorKind::UnexpectedToken(token))),
        }
    }

    /// Parse alpha component (0-1 or percentage)
    fn parse_alpha_component<'i>(&self, parser: &mut Parser<'i, '_>) -> std::result::Result<u8, ParseError<'i, ()>> {
        parser.skip_whitespace();
        let token = parser.next()?.clone();
        match token {
            Token::Number { value, .. } => Ok((value.clamp(0.0, 1.0) * 255.0) as u8),
            Token::Percentage { unit_value, .. } => Ok((unit_value * 255.0).clamp(0.0, 255.0) as u8),
            _ => Err(parser.new_error(BasicParseErrorKind::UnexpectedToken(token))),
        }
    }

    /// Skip to the next rule (after closing brace)
    fn skip_to_next_rule<'i>(&self, parser: &mut Parser<'i, '_>) -> std::result::Result<(), ParseError<'i, ()>> {
        loop {
            match parser.next() {
                Ok(Token::CurlyBracketBlock) => {
                    let _ = parser.parse_nested_block(|p| {
                        while p.next().is_ok() {}
                        Ok::<(), ParseError<()>>(())
                    });
                    break;
                }
                Err(_) => break,
                _ => continue,
            }
        }
        Ok(())
    }

    /// Skip to next semicolon or end of block
    fn skip_to_semicolon<'i>(&self, parser: &mut Parser<'i, '_>) -> std::result::Result<(), ParseError<'i, ()>> {
        loop {
            match parser.next() {
                Ok(Token::Semicolon) => break,
                Err(_) => break,
                _ => continue,
            }
        }
        Ok(())
    }
}

impl Default for CssParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let parser = CssParser::new();
        let css = "body { color: red; }";
        let stylesheet = parser.parse(css).unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors[0].tag_name, Some("body".to_string()));
        assert_eq!(stylesheet.rules[0].declarations.len(), 1);
        assert_eq!(stylesheet.rules[0].declarations[0].property, "color");
    }

    #[test]
    fn test_parse_hex_color() {
        let parser = CssParser::new();
        let css = "div { background: #ff0000; }";
        let stylesheet = parser.parse(css).unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
        if let Value::Color(c) = &stylesheet.rules[0].declarations[0].value {
            assert_eq!(c.r, 255);
            assert_eq!(c.g, 0);
            assert_eq!(c.b, 0);
        } else {
            panic!("Expected color value");
        }
    }

    #[test]
    fn test_parse_length() {
        let parser = CssParser::new();
        let css = "p { margin: 10px; }";
        let stylesheet = parser.parse(css).unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
        if let Value::Length(val, unit) = &stylesheet.rules[0].declarations[0].value {
            assert_eq!(*val, 10.0);
            assert_eq!(*unit, Unit::Px);
        } else {
            panic!("Expected length value");
        }
    }

    #[test]
    fn test_parse_class_selector() {
        let parser = CssParser::new();
        let css = ".container { width: 100%; }";
        let stylesheet = parser.parse(css).unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
        assert!(stylesheet.rules[0].selectors[0].classes.contains(&"container".to_string()));
    }

    #[test]
    fn test_parse_id_selector() {
        let parser = CssParser::new();
        let css = "#main { height: 50vh; }";
        let stylesheet = parser.parse(css).unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors[0].id, Some("main".to_string()));
    }

    #[test]
    fn test_parse_multiple_declarations() {
        let parser = CssParser::new();
        let css = "div { color: blue; font-size: 16px; margin: 10px; }";
        let stylesheet = parser.parse(css).unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].declarations.len(), 3);
    }

    #[test]
    fn test_parse_rgb_color() {
        let parser = CssParser::new();
        let css = "span { color: rgb(128, 64, 32); }";
        let stylesheet = parser.parse(css).unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
        if let Value::Color(c) = &stylesheet.rules[0].declarations[0].value {
            assert_eq!(c.r, 128);
            assert_eq!(c.g, 64);
            assert_eq!(c.b, 32);
        } else {
            panic!("Expected color value");
        }
    }
}

