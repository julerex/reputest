# Use the official Rust image as the base image
FROM rust:1.75-slim as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release

# Remove the dummy main.rs and copy the actual source code
RUN rm src/main.rs
COPY src ./src

# Build the actual application
RUN cargo build --release

# Use a minimal runtime image
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS requests
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false appuser

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/reputest /usr/local/bin/reputest

# Change ownership of the binary
RUN chown appuser:appuser /usr/local/bin/reputest

# Switch to the non-root user
USER appuser

# Expose the port the app runs on
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV PORT=3000

# Run the application
CMD ["reputest"]
