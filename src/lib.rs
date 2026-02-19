use std::io::{Cursor, Read};
use wasm_bindgen::prelude::*;
use xml2rdf::convert::parse_xml;
use xml2rdf::writer::FileWriter as Xml2RdfFileWriter;
use json2rdf;
use json2rdf::writer::FileWriter as Json2RdfFileWriter;

pub fn xml_to_ttl(xml: &str, base: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Use in-memory reader and writer so this works in wasm
    let data = xml.as_bytes().to_vec();
    let cursor = Cursor::new(data);
    let readers: Vec<Box<dyn Read>> = vec![Box::new(cursor)];

    let mut writer = Xml2RdfFileWriter::to_vec();
    parse_xml(readers, &mut writer, base).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let buf = writer.into_vec();
    let s = String::from_utf8(buf)?;
    Ok(s)
}

pub fn json_to_ttl(json: &str, base: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Use in-memory reader and writer so this works in wasm
    let data = json.as_bytes();
    let cursor = Cursor::new(data);

    let mut writer = Json2RdfFileWriter::to_vec();
    json2rdf::json_to_rdf(cursor, &mut writer, &Some(base.to_string())).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let buf = writer.into_vec();
    let s = String::from_utf8(buf)?;
    Ok(s)
}

pub fn apply_replacements(
    ttl: String,
    new_base_uri: &str,
    xml_namespace: Option<&str>,
) -> String {
    const OLD_NAMESPACE: &str = "https://decisym.ai";
    let base_uri_without_hash = new_base_uri.trim_end_matches('#');
    
    let mut result = ttl;
    
    if let Some(namespace) = xml_namespace {
        // Pattern 1: decisym.ai#/ -> base#namespace.
        result = result.replace(
            &format!("{}#/", OLD_NAMESPACE),
            &format!("{}#{}.", base_uri_without_hash, namespace),
        );

        // Pattern 2: decisym.ai/xml2rdf/model# -> base/xml2rdf/model#namespace.
        result = result.replace(
            &format!("{}/xml2rdf/model#", OLD_NAMESPACE),
            &format!("{}/xml2rdf/model#{}.", base_uri_without_hash, namespace),
        );

        // Pattern 3: Replace any remaining old namespace
        result = result.replace(OLD_NAMESPACE, base_uri_without_hash);

        // Pattern 4: Clean up any remaining #/
        result = result.replace("#/", "#");
    } else {
        result = result.replace(OLD_NAMESPACE, base_uri_without_hash);
    }
    
    result
}

#[wasm_bindgen]
pub fn convert_xml_to_ttl(
    bytes: &[u8],
    base_uri: &str,
    xml_namespace: Option<String>,
) -> Result<String, JsValue> {
    let xml = std::str::from_utf8(bytes)
        .map_err(|_| JsValue::from_str("Invalid UTF-8"))?;
    
    let ttl = xml_to_ttl(xml, base_uri)
        .map_err(|e| JsValue::from_str(&format!("Conversion failed: {}", e)))?;
    
    let processed = apply_replacements(
        ttl,
        base_uri,
        xml_namespace.as_deref(),
    );
    
    Ok(processed)
}

#[wasm_bindgen]
pub fn convert_json_to_ttl(
    bytes: &[u8],
    base_uri: &str,
    json_namespace: Option<String>,
) -> Result<String, JsValue> {
    let json = std::str::from_utf8(bytes)
        .map_err(|_| JsValue::from_str("Invalid UTF-8"))?;
    
    let ttl = json_to_ttl(json, base_uri)
        .map_err(|e| JsValue::from_str(&format!("Conversion failed: {}", e)))?;
    
    let processed = apply_replacements(
        ttl,
        base_uri,
        json_namespace.as_deref(),
    );
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_to_ttl_escaping() {
        let xml = r#"<root><name>He said \"Hi\" &amp; more</name></root>"#;
        let ttl = xml_to_ttl(xml, "https://example.com/#").expect("conversion");
        assert!(!ttl.is_empty());
        assert!(ttl.contains("He said"));
        assert!(ttl.contains("Hi"));
        assert!(ttl.contains("& more"));
    }

    #[test]
    fn test_json_to_ttl_simple() {
        let json = r#"{"name": "test", "value": 42}"#;
        let ttl = json_to_ttl(json, "https://example.com/").expect("conversion");
        assert!(!ttl.is_empty());
        assert!(ttl.contains("test"));
        assert!(ttl.contains("42"));
    }

    #[test]
    fn test_namespace_replacement_with_xml_namespace() {
        let ttl = r#"
<https://decisym.ai#/item1> <https://decisym.ai/xml2rdf/model#hasValue> "test" .
<https://decisym.ai#/item2> <https://decisym.ai/xml2rdf/model#hasName> "name" .
        "#.to_string();

        let result = apply_replacements(
            ttl,
            "https://example.org#",
            Some("ecospold02"),
        );

        assert!(!result.contains("decisym.ai"));
        assert!(result.contains("https://example.org#ecospold02.item1"));
        assert!(result.contains("https://example.org/xml2rdf/model#ecospold02.hasValue"));
    }

    #[test]
    fn test_namespace_replacement_without_xml_namespace() {
        let ttl = r#"
<https://decisym.ai#/item1> <https://decisym.ai/xml2rdf/model#hasValue> "test" .
        "#.to_string();

        let result = apply_replacements(
            ttl,
            "https://example.org#",
            None,
        );

        assert!(!result.contains("decisym.ai"));
        assert!(result.contains("https://example.org"));
    }

    #[test]
    fn test_multiple_elements_count() {
        let xml = r#"<r><a>1</a><b>2</b><c></c><d>   </d><e>3</e></r>"#;
        let ttl = xml_to_ttl(xml, "https://ex/#").expect("conversion");
        assert!(ttl.contains("\"1\""));
        assert!(ttl.contains("\"2\""));
        assert!(ttl.contains("\"3\""));
    }

    #[test]
    fn test_ecospold02_sample_conversion() {
        let path = format!(
            "{}/../dataset/xml/ecospold02/generated_ecospold2_10.xml",
            env!("CARGO_MANIFEST_DIR")
        );
        let xml = std::fs::read_to_string(&path)
            .expect(&format!("Failed to read test XML file at path: {}", path));

        let res = xml_to_ttl(&xml, "https://example.org/#");
        assert!(res.is_ok());
        let ttl = res.unwrap();
        assert!(!ttl.trim().is_empty());
        
        let processed = apply_replacements(
            ttl,
            "https://purl.org/wiser#",
            Some("ecospold02"),
        );
        assert!(!processed.contains("decisym.ai"));
    }
}