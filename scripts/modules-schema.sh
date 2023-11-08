#!/usr/bin/env bash

# Generates the schemas for each module and copies them to ./schema/abstract/<contract-name>/<version>
version="cargo pkgid | cut -d@ -f2"
SCHEMA_OUT_DIR=$(echo "$PWD"/schema)

cd ./modules

# Generates schemas for each contract, removes the "Raw" schema, and copies the rest to the schema output directory.
SCHEMA_OUT_DIR=$SCHEMA_OUT_DIR version=$version \
cargo ws exec --no-bail bash -lc \
'cargo schema && \
{ tmp=$(mktemp); jq ".contract_version = \"$(eval $version)\"" schema/module-schema.json > "$tmp" && mv "$tmp" schema/module-schema.json; \
rm -rf "schema/raw"; outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*/}/$(eval $version)"; mkdir -p "$outdir"; cp -a "schema/." "$outdir";}'
