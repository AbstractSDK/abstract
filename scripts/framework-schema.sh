#!/usr/bin/env bash

# Generates the schemas for each contract and copies them to ./schema/abstract/<contract-name>/<version>
SCHEMA_OUT_DIR=$(echo "$PWD"/schema)

cd ./framework/

VERSION=$(grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}');

# Generates schemas for each contract, removes the "Raw" schema, and copies the rest to the schema output directory.
SCHEMA_OUT_DIR=$SCHEMA_OUT_DIR VERSION=$VERSION \
cargo ws exec --no-bail bash -lc \
'cargo schema && { rm -rf "schema/raw"; outdir="$SCHEMA_OUT_DIR/${PWD##*/}/$VERSION"; mkdir -p "$outdir"; cp -a "schema/." "$outdir";}'
