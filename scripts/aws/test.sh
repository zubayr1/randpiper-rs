killall -9 node-bft
timeout 360 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/d100-n32/nodes-$1.dat -d 70 -i ./randpiper-rs/ips_file > output.log
