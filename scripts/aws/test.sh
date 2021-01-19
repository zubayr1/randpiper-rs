killall -9 node-bft
timeout 60 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/d100-n16/nodes-$1.dat -d 80 -i ./randpiper-rs/ips_file > output.log
