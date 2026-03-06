#!/usr/bin/env bash
#
# apply-freya-patches.sh
#
# Applies pre-publish patches to rename crates and update URLs.
# Safe to run multiple times (idempotent).
#
# What this does:
#   - Renames package names:  skia-safe        -> freya-skia-safe
#                             skia-bindings    -> freya-skia-bindings
#                             skia-svg-macros  -> freya-skia-svg-macros
#   - Preserves Rust identifiers (skia_bindings, skia_safe, skia_svg_macros)
#     via [lib] name = and package = aliases — NO source files are touched.
#   - Updates binary download URL to marc2332/rust-skia
#   - Updates homepage/repository metadata to marc2332/rust-skia
#
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"

# ---------------------------------------------------------------------------
# 1. Binary download URL
# ---------------------------------------------------------------------------
echo "==> Patching binary download URL..."
sed -i 's|rust-skia/skia-binaries/releases/download|marc2332/rust-skia/releases/download|g' \
    "$ROOT/skia-bindings/build_support/binary_cache/env.rs"

# ---------------------------------------------------------------------------
# 2. Homepage / repository metadata
# ---------------------------------------------------------------------------
echo "==> Patching homepage/repository URLs..."
for f in \
    "$ROOT/skia-safe/Cargo.toml" \
    "$ROOT/skia-bindings/Cargo.toml" \
    "$ROOT/skia-svg-macros/Cargo.toml"
do
    sed -i 's|https://github.com/rust-skia/rust-skia|https://github.com/marc2332/rust-skia|g' "$f"
done

# ---------------------------------------------------------------------------
# 3. Rename package names (the crates.io-visible name)
# ---------------------------------------------------------------------------
echo "==> Renaming package names..."
sed -i 's/^name = "skia-safe"$/name = "freya-skia-safe"/'             "$ROOT/skia-safe/Cargo.toml"
sed -i 's/^name = "skia-bindings"$/name = "freya-skia-bindings"/'     "$ROOT/skia-bindings/Cargo.toml"
sed -i 's/^name = "skia-svg-macros"$/name = "freya-skia-svg-macros"/' "$ROOT/skia-svg-macros/Cargo.toml"

# ---------------------------------------------------------------------------
# 4. Preserve Rust identifiers via [lib] name =
#    Without this, rustc would derive the crate name from the package name,
#    breaking all `use skia_bindings::...` and `use skia_svg_macros::...`
#    throughout the source without touching a single .rs file.
# ---------------------------------------------------------------------------
echo "==> Pinning [lib] names to preserve Rust identifiers..."
# skia-bindings: insert `name = "skia_bindings"` into its [lib] section
sed -i '/^\[lib\]/{n; /^name = /!s/^\(doctest\)/name = "skia_bindings"\n\1/}' \
    "$ROOT/skia-bindings/Cargo.toml"
# skia-svg-macros: insert `name = "skia_svg_macros"` into its [lib] section
sed -i '/^\[lib\]/{n; /^name = /!s/^\(proc-macro\)/name = "skia_svg_macros"\n\1/}' \
    "$ROOT/skia-svg-macros/Cargo.toml"
# skia-safe: insert `name = "skia_safe"` into its [lib] section
sed -i '/^\[lib\]/{n; /^name = /!s/^\(doctest\)/name = "skia_safe"\n\1/}' \
    "$ROOT/skia-safe/Cargo.toml"

# ---------------------------------------------------------------------------
# 5. Update dependency declarations in skia-safe/Cargo.toml to use
#    package = "freya-skia-*" aliases, keeping the local alias unchanged.
#    This means `skia-bindings/gl` feature refs and `use skia_bindings::...`
#    in source all continue to work without modification.
# ---------------------------------------------------------------------------
echo "==> Patching dependency declarations in skia-safe/Cargo.toml..."
# skia-bindings dep: add package = "freya-skia-bindings" (only if not already present)
sed -i 's/^skia-bindings = { version = \(.*\), path = "\(.*\)", default-features = false }$/skia-bindings = { package = "freya-skia-bindings", version = \1, path = "\2", default-features = false }/' \
    "$ROOT/skia-safe/Cargo.toml"
# skia-svg-macros dep: add package = "freya-skia-svg-macros"
sed -i 's/^skia-svg-macros = { version = \(.*\), path = "\(.*\)", optional = true }$/skia-svg-macros = { package = "freya-skia-svg-macros", version = \1, path = "\2", optional = true }/' \
    "$ROOT/skia-safe/Cargo.toml"

echo ""
echo "Done. Summary:"
echo "  - Binary URL:    rust-skia/skia-binaries -> marc2332/rust-skia"
echo "  - Package names: skia-safe -> freya-skia-safe"
echo "                   skia-bindings -> freya-skia-bindings"
echo "                   skia-svg-macros -> freya-skia-svg-macros"
echo "  - Rust identifiers preserved (skia_bindings, skia_safe, skia_svg_macros)"
echo "  - No .rs source files modified"
echo ""
echo "NOTE: These changes are NOT committed. Re-run at any time."
