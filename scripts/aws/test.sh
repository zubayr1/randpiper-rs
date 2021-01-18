killall -9 node-bft
timeout 600 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/d100-n10/nodes-$1.json -d 150 -i ./randpiper-rs/ips_file > output.log
