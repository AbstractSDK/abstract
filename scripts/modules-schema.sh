#!/usr/bin/env bash
cd ./modules

version="cargo pkgid | cut -d@ -f2"
SCHEMA_OUT_DIR=$(cd ../schemas && echo "$PWD")

cargo ws exec --no-bail bash -lc \
'cargo schema && \
{ tmp=$(mktemp); jq ".contract_version = \"$(eval $version)\"" schema/module-schema.json > "$tmp" && mv "$tmp" schema/module-schema.json; \
rm -rf "schema/raw"; outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*}/$(eval $version); mkdir -p "$outdir"; cp -a "schema/." "$outdir";}'
