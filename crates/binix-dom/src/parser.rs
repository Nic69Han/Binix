use html5ever::parse_document;
use markup5ever_rcdom::RcDom;
use std::io::Cursor;
use binix_core::Result;

pub struct HtmlParser;

impl HtmlParser {
    pub fn parse(html: &str) -> Result<RcDom> {
        parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut Cursor::new(html.as_bytes()))
            .map_err(|e| binix_core::error::BinixError::Parse(e.to_string()))
    }
}
