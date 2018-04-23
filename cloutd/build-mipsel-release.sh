#!/bin/bash
cargo build --target=mipsel-unknown-linux-musl --release
mipsel-openwrt-linux-musl-strip target/mipsel-unknown-linux-musl/release/cloutd
