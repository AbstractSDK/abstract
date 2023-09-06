# go into the directory we want to compile
cd ./modules

# Create lock file if it does not exist
if [ ! -f Cargo.lock ]; then
  cargo generate-lockfile
fi

# Check the installed version
go version

cargo llvm-cov --locked --lcov --output-path lcov.info

# print the result.
ls -la .