// Document parser — Extract text from PDF, DOCX, TXT files

use std::io::Read;

pub struct ParsedDocument {
    pub text: String,
    pub metadata: DocumentMetadata,
}

pub struct DocumentMetadata {
    pub page_count: Option<usize>,
    #[allow(dead_code)]
    pub title: Option<String>,
}

/// Parse a document and extract text content
pub fn parse_document(file_path: &str, file_type: &str) -> Result<ParsedDocument, String> {
    match file_type {
        "pdf" => parse_pdf(file_path),
        "docx" => parse_docx(file_path),
        "txt" | "md" | "csv" | "text" | "markdown" => parse_plain_text(file_path),
        _ => parse_plain_text(file_path),
    }
}

/// Parse PDF using lopdf
fn parse_pdf(file_path: &str) -> Result<ParsedDocument, String> {
    let doc = lopdf::Document::load(file_path)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;

    let pages = doc.get_pages();
    let page_count = pages.len();

    // Collect all page numbers for extraction
    let page_nums: Vec<u32> = pages.keys().copied().collect();

    let text = doc
        .extract_text(&page_nums)
        .map_err(|e| format!("Failed to extract PDF text: {}", e))
        .unwrap_or_default();

    Ok(ParsedDocument {
        text,
        metadata: DocumentMetadata {
            page_count: Some(page_count),
            title: None,
        },
    })
}

/// Parse DOCX using zip + quick-xml
fn parse_docx(file_path: &str) -> Result<ParsedDocument, String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open DOCX: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read DOCX as ZIP: {}", e))?;

    let mut xml_content = String::new();
    {
        let mut document_xml = archive
            .by_name("word/document.xml")
            .map_err(|e| format!("Failed to find word/document.xml in DOCX: {}", e))?;
        document_xml
            .read_to_string(&mut xml_content)
            .map_err(|e| format!("Failed to read document.xml: {}", e))?;
    }

    let text = extract_docx_text(&xml_content)?;

    Ok(ParsedDocument {
        text,
        metadata: DocumentMetadata {
            page_count: None,
            title: None,
        },
    })
}

/// Extract text from DOCX XML by parsing <w:t> tags
fn extract_docx_text(xml: &str) -> Result<String, String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    let mut text = String::new();
    let mut in_text_element = false;
    let mut in_paragraph = false;
    let mut paragraph_has_text = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                match name {
                    "t" => in_text_element = true,
                    "p" => {
                        if in_paragraph && paragraph_has_text {
                            text.push('\n');
                        }
                        in_paragraph = true;
                        paragraph_has_text = false;
                    }
                    "br" => text.push('\n'),
                    "tab" => text.push('\t'),
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                match name {
                    "t" => in_text_element = false,
                    "p" => {
                        in_paragraph = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text_element {
                    let decoded = e.unescape().map_err(|e| format!("XML decode error: {}", e))?;
                    text.push_str(&decoded);
                    paragraph_has_text = true;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
    }

    // Add final newline if there was text
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }

    Ok(text)
}

/// Parse plain text files (TXT, MD, CSV)
fn parse_plain_text(file_path: &str) -> Result<ParsedDocument, String> {
    let text = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    Ok(ParsedDocument {
        text,
        metadata: DocumentMetadata {
            page_count: None,
            title: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_docx_text_basic() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
          <w:body>
            <w:p><w:r><w:t>Hello World</w:t></w:r></w:p>
            <w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p>
          </w:body>
        </w:document>"#;

        let text = extract_docx_text(xml).unwrap();
        assert!(text.contains("Hello World"));
        assert!(text.contains("Second paragraph"));
    }

    #[test]
    fn test_parse_plain_text() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "Hello, World!").unwrap();

        let result = parse_document(
            tmp.path().to_str().unwrap(),
            "txt",
        )
        .unwrap();

        assert_eq!(result.text, "Hello, World!");
        assert!(result.metadata.page_count.is_none());
    }
}
