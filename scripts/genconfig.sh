set -e

cargo build --package=genconfig
mkdir -p ./test/config
./target/debug/genconfig -n 4 -d 4000 -b 1 -C 15000 -P 16000 -t ./test/config/