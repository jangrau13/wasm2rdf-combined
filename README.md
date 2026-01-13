# wasm2rdf

WASM bridge for xml2rdf and json2rdf: stream-to-stream XML/JSON -> RDF conversion

This crate provides `wasm-bindgen` functions to convert XML and JSON strings into RDF Turtle format. ZIP handling should be done on the JavaScript side (unpack the ZIP and pass XML/JSON strings to the functions). It integrates the full `xml2rdf` and `json2rdf` parsers for complete conversion logic.

Build (requires `wasm-pack`):

```bash
cd wasm2rdf
wasm-pack build --target web --out-dir pkg --release
```

Example usage in the browser (after bundling):

```js
import init, { convert_xml_to_ttl, convert_json_to_ttl } from './pkg/wasm2rdf.js';
await init();

// Convert XML
const xmlTtl = convert_xml_to_ttl(xmlBytes, 'https://example.com#', 'namespace');

// Convert JSON
const jsonTtl = convert_json_to_ttl(jsonBytes, 'https://example.com#', 'namespace');
```

For Node use `--target nodejs` with `wasm-pack`.

Notes:
- This implementation integrates the full `xml2rdf` and `json2rdf` conversion logic for complete RDF generation.
- Supports namespace replacement for custom base URIs and optional XML/JSON namespaces.
