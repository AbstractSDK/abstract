#!/usr/bin/env bash
version="cargo pkgid | cut -d@ -f2"
SCHEMA_OUT_DIR=$(cd ../schemas && echo "$PWD")

cd ./framework

SCHEMA_OUT_DIR=$SCHEMA_OUT_DIR version=$version \
cargo ws exec --no-bail bash -lc \
'cargo schema && { rm -rf "schema/raw"; outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*/}/$(eval $version)"; mkdir -p "$outdir"; cp -a "schema/." "$outdir";}'
