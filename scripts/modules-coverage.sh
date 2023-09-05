# create a dummy container which will hold a volume with config
# docker create -v /code -v /integrations -v /framework --name modules_with_code alpine /bin/true

# # copy directories to container.
# docker cp ./integrations modules_with_code:/
# docker cp ./framework modules_with_code:/

# go into the directory we want to compile
cd ./modules

# Install cargo-llvm-cov for coverage generation
cargo install cargo-llvm-cov

# Create lock file if it does not exist
if [ ! -f Cargo.lock ]; then
  cargo generate-lockfile
fi

# Install go for test-tube
rm -rf /usr/local/go && tar -C /usr/local -xzf go1.21.0.linux-amd64.tar.gz
export PATH=$PATH:/usr/local/go/bin
go version

cargo llvm-cov --locked --all-features --lcov --output-path lcov.info

# print the result
ls -la .