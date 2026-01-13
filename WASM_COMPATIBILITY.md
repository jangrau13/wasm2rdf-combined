# WASM Compatibility Notes: getrandom and oxrdf

## Problem

When building Rust crates for WebAssembly (`wasm32-unknown-unknown` target), you may encounter compilation errors related to `getrandom` version incompatibility, especially when using crates that depend on `oxrdf` for RDF graph operations.

### Error Symptoms

```
error: The "wasm_js" backend requires the `wasm_js` feature for `getrandom`. 
For more information see: https://docs.rs/getrandom/0.3.4/#webassembly-support
```

Or:

```
error: The wasm32-unknown-unknown targets are not supported by default; 
you may need to enable the "js" feature.
```

## Root Cause

The issue stems from a dependency chain conflict:

1. **oxrdf 0.3.x** depends on newer versions of `rand`, which pulls in **getrandom 0.3.x**
2. **getrandom 0.3.x** requires the `wasm_js` feature to work with `wasm32-unknown-unknown` target
3. **getrandom 0.2.x** (older) also works but is not needed if you enable the feature properly

### Dependency Chain

**Before (getrandom 0.3 without feature):**
```
oxrdf 0.3.1 → uuid → rand → getrandom 0.3.x ❌ (WASM fails without wasm_js feature)
```

**After (getrandom 0.3.4 with wasm_js feature):**
```
oxrdf 0.3.1 → uuid → rand → getrandom 0.3.4 ✅ (WASM works with wasm_js feature)
```

## Solution

### Option 1: Use Latest Versions (Recommended)

Use the latest oxrdf 0.3.x with getrandom 0.3.4 and the `wasm_js` feature:

```toml
[dependencies]
oxrdf = "0.3"
serde_json = "1.0.148"

[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.3.4", features = ["wasm_js"] }
```

Build with:
```bash
export RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\""
cargo build --target wasm32-unknown-unknown --release
```

### Option 2: Use Older Stable Versions (Legacy)

If you need to use older dependencies, use oxrdf 0.2.x with getrandom 0.2.x:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

This ensures:
- **Non-WASM builds**: Use whatever getrandom version the dependencies specify
- **WASM builds**: Override to use getrandom 0.2 with the "js" feature enabled

### 3. Set RUSTFLAGS when building for WASM

When building for WASM, set the proper backend flag:

```bash
export RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\""
cargo build --target wasm32-unknown-unknown --release
```

Or with wasm-bindgen:

```bash
export RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\""
wasm-bindgen target/wasm32-unknown-unknown/release/your_crate.wasm \
  --out-dir pkg \
  --target bundler
```

## Complete Example

### json2rdf (Simple case)

See the `json2rdf/Cargo.toml` in this repository for a working example:

```toml
[dependencies]
blake3 = "1.0"
oxrdf = "0.3"           # Use latest
serde_json = "1.0.148"
urlencoding = "2.1"

[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.3.4", features = ["wasm_js"] }  # Enable wasm_js feature
```

### xml2rdf (With target-specific feature flags)

For crates that depend on libraries with WASM-incompatible features (like `uuid`), use target-specific overrides:

```toml
[dependencies]
const_format = "0.2"
oxrdf = "0.3"
uuid = { version = "1.15", features = ["v4"] }  # Base features only
xml-rs = "0.8"

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
# Native build: use all features for performance
uuid = { version = "1.15", features = ["v4", "fast-rng", "macro-diagnostics"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
# WASM build: use js feature for RNG, add getrandom with wasm_js
uuid = { version = "1.15", features = ["v4", "js"] }
getrandom = { version = "0.3.4", features = ["wasm_js"] }
```

**Key pattern**: When a dependency has WASM-incompatible features:
1. Put minimal base features in `[dependencies]`
2. Use `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` for native-only features
3. Use `[target.'cfg(target_arch = "wasm32")'.dependencies]` for WASM-specific features

## Testing

Verify both native and WASM builds work:

```bash
# Native build (should work without getrandom override)
cargo check

# WASM build (requires getrandom 0.3.4 with wasm_js feature)
RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\"" cargo check --target wasm32-unknown-unknown
```

## When to Use oxrdf 0.2 vs 0.3

- **Use oxrdf 0.3**: Recommended for all new projects. Works with WASM when getrandom 0.3.4 is configured with the `wasm_js` feature
- **Use oxrdf 0.2**: Only if you have compatibility requirements with legacy systems or can't use the latest dependencies

## References

- [getrandom WebAssembly Support](https://docs.rs/getrandom/#webassembly-support)
- [oxrdf Releases](https://github.com/oxigraph/oxrdf/releases)
- [WASM Bindgen Documentation](https://rustwasm.org/)

## Related Issues

This pattern applies to any Rust crate that:
- Depends on `oxrdf >= 0.3` 
- Needs to compile to `wasm32-unknown-unknown`
- Uses dependencies like `rand`, `uuid`, or other crates that depend on `getrandom`

## Troubleshooting

### Debugging Transitive Dependencies

If your WASM build fails with getrandom errors, identify what's pulling in the conflicting version:

```bash
# Check WASM target dependency tree
cargo tree --target wasm32-unknown-unknown 2>&1 | grep -B 5 "getrandom v0.2"
```

This will show you which crate is pulling in the problematic version.

### Common Causes

1. **Unused dependencies** - Remove dependencies no longer needed (e.g., old `oxrdfio` when moving to oxrdf 0.3)
   ```bash
   cargo tree --duplicates  # Find duplicate versions
   ```

2. **Features with WASM incompatibilities** - Use target-specific features (see xml2rdf example above)

3. **Transitive dependencies** - Sometimes a dependency you don't directly use (like `oxrdfio`) pulls in incompatible versions. Use `cargo tree` to trace the chain and consider removing or updating the intermediate dependency.
