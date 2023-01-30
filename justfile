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
  ./publish/publish.sh