build:
  cargo build

# Test everything
test: build
  cargo nextest run

lint:
  cargo clippy -- -D warnings

lintfix:
  cargo clippy --fix --allow-staged --allow-dirty

refresh:
  cargo clean && cargo update

check-codecov:
  cat codecov.yml | curl --data-binary @- https://codecov.io/validate