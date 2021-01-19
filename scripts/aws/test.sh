killall -9 node-bft
timeout 180 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/d100-n16/nodes-$1.dat -d 50 -i ./randpiper-rs/ips_file > output.log
