killall -9 node-bft
timeout 600 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/d100-n32/nodes-$1.dat -d 280 -i ./randpiper-rs/ips_file > output.log
