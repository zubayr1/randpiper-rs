set -e
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

cargo build --package=node-bft
./target/debug/node-bft -c ./target/config/nodes-0.json -i ./scripts/ip_file &
./target/debug/node-bft -c ./target/config/nodes-1.json -i ./scripts/ip_file &
./target/debug/node-bft -c ./target/config/nodes-2.json -i ./scripts/ip_file &
./target/debug/node-bft -c ./target/config/nodes-3.json -i ./scripts/ip_file &

wait
