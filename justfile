build:
  cargo build

# Test everything
test:
  cargo nextest run

format:
  cargo fmt --all

lint:
  cargo clippy --all -- -D warnings
#  cargo clippy --all --all-targets --all-features -- -D warnings

lintfix:
  cargo clippy --fix --allow-staged --allow-dirty

refresh:
  cargo clean && cargo update

check-codecov:
  cat codecov.yml | curl --data-binary @- https://codecov.io/validate

publish:
  set -e
  git tag v`grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}'`
  git push origin v`grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}'`

watch:
  cargo watch -x lcheck

watch-test:
  cargo watch -x "nextest run"
