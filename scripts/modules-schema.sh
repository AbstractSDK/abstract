#!/usr/bin/env bash
cd ./modules

version="cargo pkgid | cut -d@ -f2"

cargo ws exec --no-bail bash -lc \
'cargo schema && \
{ tmp=$(mktemp); jq ".contract_version = \"$(eval $version)\"" schema/module-schema.json > "$tmp" && mv "$tmp" schema/module-schema.json; \
rm -rf "schema/raw";}'
