set -e
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

TYPE=${TYPE:-"release"}
TESTDIR=${TESTDIR:-"./test/config"}

cargo build --package=node-bft --release
./target/$TYPE/node-bft -c $TESTDIR/nodes-0.json -i ./scripts/ip_file $1 &> 0.log&
./target/$TYPE/node-bft -c $TESTDIR/nodes-1.json -i ./scripts/ip_file $1 &> 1.log&
./target/$TYPE/node-bft -c $TESTDIR/nodes-2.json -i ./scripts/ip_file $1 &> 2.log&
./target/$TYPE/node-bft -c $TESTDIR/nodes-3.json -i ./scripts/ip_file $1 &> 3.log&

wait
