# go into the directory we want to compile
cd ./modules

# Install cargo-llvm-cov for coverage generation
# Get host target
host=$(rustc -Vv | grep host | sed 's/host: //')
# Download binary and install to $HOME/.cargo/bin
curl -LsSf https://github.com/taiki-e/cargo-llvm-cov/releases/latest/download/cargo-llvm-cov-$host.tar.gz | tar xzf - -C $HOME/.cargo/bin

# Create lock file if it does not exist
if [ ! -f Cargo.lock ]; then
  cargo generate-lockfile
fi

sudo apt-get update && sudo apt-get install libclang-dev -y

# Set Go version
GO_VERSION="1.18"

# Download Go
wget https://dl.google.com/go/go${GO_VERSION}.linux-amd64.tar.gz -O /tmp/go${GO_VERSION}.linux-amd64.tar.gz

# Extract Go archive
sudo tar -C /usr/local -xzf /tmp/go${GO_VERSION}.linux-amd64.tar.gz

# Set environment variables
echo "export GOROOT=/usr/local/go" >> ~/.bash_profile
echo "export GOPATH=$HOME/go" >> ~/.bash_profile
echo "export PATH=$PATH:/usr/local/go/bin:$GOPATH/bin" >> ~/.bash_profile

cat ~/.bash_profile

# Load the environment variables
source ~/.bash_profile

# Check the installed version
go version

cargo llvm-cov --locked --all-features --lcov --output-path lcov.info

# print the result
ls -la .