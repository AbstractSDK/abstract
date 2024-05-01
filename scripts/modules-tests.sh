# go into the directory we want to compile
cd ./modules

# Create lock file if it does not exist
if [ ! -f Cargo.lock ]; then
  cargo generate-lockfile
fi

# Force recompile osmosis test tube so build.rs is runned
cargo clean -p osmosis-test-tube

sudo apt-get update && sudo apt-get install libclang-dev -y

# Set Go version
GO_VERSION="1.21.9"

# Download Go
wget https://dl.google.com/go/go${GO_VERSION}.linux-amd64.tar.gz -O /tmp/go${GO_VERSION}.linux-amd64.tar.gz

# Extract Go archive
sudo tar -C /usr/local -xzf /tmp/go${GO_VERSION}.linux-amd64.tar.gz

# Set environment variables
echo "export GOROOT=/usr/local/go" >> ~/.bash_profile
echo "export GOPATH=$HOME/go" >> ~/.bash_profile
echo "export PATH=$PATH:/usr/local/go/bin:$GOPATH/bin" >> ~/.bash_profile

# Install nextest
curl -LsSf https://get.nexte.st/0.9.53/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin

# Load the environment variables
source ~/.bash_profile

# Check the installed versions
go version
cargo nextest -V

cargo nextest run --locked --all-features --all-targets --build-jobs 3