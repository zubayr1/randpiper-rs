killall -9 node-bft
timeout 1200 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/d500-n25/nodes-$1.json -d 600 -i ./randpiper-rs/ips_file > output.log
