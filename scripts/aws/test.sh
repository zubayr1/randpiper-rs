killall -9 node-bft
timeout 180 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/d100-n32/nodes-$1.dat -d 200 -i ./randpiper-rs/ips_file > output.log
