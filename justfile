alias b := build
alias r := release
alias t := test

build:
    cargo build --workspace --target-dir bin

release:
    cargo build --workspace --release --target-dir bin

test: build
    cargo test

clean:
    cargo clean
