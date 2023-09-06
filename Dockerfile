FROM --platform=linux/amd64 rust:1.72 as base

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    gcc \
    libclang-dev \
    ca-certificates \
    --no-install-recommends \
    && rm -r /var/lib/apt/lists/*

# Set Go version
ENV GO_VERSION=1.18

# Download Go and install it to /usr/local
RUN wget -q https://dl.google.com/go/go${GO_VERSION}.linux-amd64.tar.gz -O /tmp/go${GO_VERSION}.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf /tmp/go${GO_VERSION}.linux-amd64.tar.gz \
    && rm /tmp/go${GO_VERSION}.linux-amd64.tar.gz

# Create .cargo/bin directory
RUN mkdir -p $HOME/.cargo/bin

# Set environment variables for Go and Cargo
ENV GOROOT=/usr/local/go
ENV GOPATH=/root/go
ENV PATH=$PATH:/usr/local/go/bin:/root/go/bin:/root/.cargo/bin


# Install cargo-llvm-cov
RUN host=$(rustc -Vv | grep host | sed 's/host: //') \
    && curl -LsSf https://github.com/taiki-e/cargo-llvm-cov/releases/latest/download/cargo-llvm-cov-$host.tar.gz | tar xzf - -C $HOME/.cargo/bin

# Set working directory
WORKDIR /code

COPY scripts/modules-coverage.sh modules-coverage.sh

RUN chmod +x modules-coverage.sh
# # Create lock file if it does not exist
# RUN if [ ! -f Cargo.lock ]; then cargo generate-lockfile; fi

# Run coverage job
CMD ["bash", "scripts/modules-coverage-local.sh"]
