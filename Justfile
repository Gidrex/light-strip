@_default:
    just --list

# Run complex QA testing
qa:
    cargo check
    cargo clippy
    
# Build release version
release:
    cargo build --release
    
# Run compiled file
run:
    ./target/release/light-strip