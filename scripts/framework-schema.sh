#!/usr/bin/env bash
cd ./framework

cargo ws exec --no-bail bash -lc \
'cargo schema && rm -rf "schema/raw"'
