cargo build --release
del ./cyhdev
upx --ultra-brute ./target/release/cyhdev -o ./cyhdev