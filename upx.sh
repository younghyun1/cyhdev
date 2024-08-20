cargo build --release
rm ./cyhdev
upx --ultra-brute ./target/release/cyhdev -o ./cyhdev
