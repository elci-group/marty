# Use the official Rust image as a base
FROM rust:1.72 as builder

# Create a new empty shell project
WORKDIR /usr/src/marty
COPY . .

# Build the application
RUN cargo install --path .

# Create a new minimal image for the final container
FROM debian:buster-slim

# Copy the built binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/marty /usr/local/bin/marty

# Set the binary as the entrypoint
ENTRYPOINT ["marty"]
