killall -9 node-bft
timeout 60 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/d100-n10/nodes-$1.json -d 500 -i ./randpiper-rs/ips_file > output.log
