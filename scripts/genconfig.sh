set -e

cargo build --package=genconfig
mkdir -p ./target/config
./target/debug/genconfig -n 4 -d 100 -b 1 -C 15000 -P 16000 -t ./target/config/