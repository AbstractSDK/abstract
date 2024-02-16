# go into the directory we want to compile
cd ./framework

# Install cargo-llvm-cov for coverage generation
# Get host target
host=$(rustc -Vv | grep host | sed 's/host: //')
# # Download binary and install to $HOME/.cargo/bin
curl -LsSf https://github.com/taiki-e/cargo-llvm-cov/releases/latest/download/cargo-llvm-cov-$host.tar.gz | tar xzf - -C $HOME/.cargo/bin

# Create lock file if it does not exist
if [ ! -f Cargo.lock ]; then
  cargo generate-lockfile
fi

cargo llvm-cov --locked --lcov --output-path lcov.info

# print the result.
ls -la .