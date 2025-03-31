# Use a minimal Rust runtime image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the entire project into the container
COPY . .

# Build the Rust application
RUN cargo build --release

# Expose the correct port
EXPOSE 7171

# Ensure direct execution of the binary
CMD ["./target/release/your-binary-name"]
