# wasm2rdf

WASM bridge for xml2rdf: stream-to-stream XML -> RDF (simple prototype)

This crate provides `wasm-bindgen` functions to convert XML strings into a small TTL-like string. ZIP handling should be done on the JavaScript side (unpack the ZIP and pass XML strings to the functions). This is a minimal prototype; the full xml2rdf parser should be plugged into `xml_to_ttl_simple` or exposed through a `Writer` trait.

Build (requires `wasm-pack`):

```bash
cd wasm2rdf
wasm-pack build --target web --out-dir pkg --release
```

Example usage in the browser (after bundling):

```js
import init, { convert_xml_strings_to_ttl, stream_xml_strings_to_ttl } from './pkg/wasm2rdf.js';
await init();
// Batch
const ttl = convert_xml_strings_to_ttl([xmlString], 'https://example.com#');

// Stream (callback receives each TTL chunk)
stream_xml_strings_to_ttl([xmlString1, xmlString2], 'https://example.com#', (chunk) => {
	console.log('chunk:', chunk);
});
```

For Node use `--target nodejs` with `wasm-pack`.

Notes:
- This implementation uses a tiny heuristic to create triples and is a placeholder for the full `xml2rdf` conversion logic.
- To integrate the real parser, replace `xml_to_ttl_simple` with calls into `xml2rdf::convert::parse_xml` and provide a MemoryWriter implementation.
