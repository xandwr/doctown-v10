# Multi-stage Dockerfile for doctown
# Builds a complete pipeline with CUDA, Python ML models, and Rust binary

# Stage 1: Rust builder - use Ubuntu 22.04 base to match runtime GLIBC
FROM ubuntu:22.04 as rust-builder

WORKDIR /build

# Install Rust and build dependencies
RUN apt-get update && \
    apt-get install -y curl build-essential pkg-config libssl-dev && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.cargo/bin:${PATH}"

# Copy Rust source
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# Build release binary
RUN cargo build --release && \
    strip target/release/doctown

# Stage 2: Final image with CUDA + Python + Rust binary
FROM nvidia/cuda:12.1.1-runtime-ubuntu22.04

# Install system dependencies
RUN apt-get update && \
    apt-get install -y \
        python3.10 \
        python3-pip \
        python3.10-venv \
        git \
        wget \
        curl \
        unzip \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create Python virtual environment
RUN python3 -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Upgrade pip
RUN pip install --no-cache-dir --upgrade pip setuptools wheel

# Copy and install Python requirements
COPY requirements.txt /opt/requirements.txt
RUN pip install --no-cache-dir -r /opt/requirements.txt

# Copy Python scripts
COPY embed_chunks.py /opt/embed_chunks.py
COPY rerank.py /opt/rerank.py
RUN chmod +x /opt/*.py

# Copy Rust binary from builder
COPY --from=rust-builder /build/target/release/doctown /usr/local/bin/doctown

# Copy entrypoint script
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Create working directory and output directory
WORKDIR /workspace
RUN mkdir -p /output

# Set environment variables
ENV PYTHONUNBUFFERED=1
ENV CUDA_VISIBLE_DEVICES=0

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD doctown --version || exit 1

# Use entrypoint script to handle URL downloads
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["--help"]
