# PP-OCR-RS Makefile

.PHONY: build run test clean help install check

# Default target
all: build

# Build the project with server feature in release mode
build:
	@echo "Building PP-OCR-RS..."
	cargo build --features server --release
	@echo "Build completed successfully!"

# Run the OCR server
run: build
	@echo "Starting OCR server..."
	./target/release/ocr serve -c config.yaml

# Run API tests
test:
	@echo "Running API tests..."
	./test_api.sh -a

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Clean completed!"

# Install dependencies
install:
	@echo "Installing dependencies..."
	cargo install --features server --path .

# Check code without building
check:
	@echo "Checking code..."
	cargo check --features server
	cargo clippy --features server -- -D warnings
	cargo fmt -- --check

# Run development server (debug mode)
dev:
	@echo "Starting development server..."
	cargo run --features server --bin ocr -- serve -c config.yaml

# Show help
help:
	@echo "PP-OCR-RS Makefile Commands:"
	@echo "  build    - Build the project with server feature in release mode"
	@echo "  run      - Build and run the OCR server"
	@echo "  test     - Run API tests"
	@echo "  clean    - Clean build artifacts"
	@echo "  install  - Install the project"
	@echo "  check    - Check code without building"
	@echo "  dev      - Run development server (debug mode)"
	@echo "  help     - Show this help message"