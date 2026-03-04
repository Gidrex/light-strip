@_default:
    just --list

# Run complex QA testing
qa: fmt
    cargo check
    cargo clippy

# Format the code
fmt:
    cargo fmt --all
    
# Build release version
release:
    cargo build --release
    
# Run compiled file
run:
    ./target/release/light-strip