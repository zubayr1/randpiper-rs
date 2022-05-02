# randpiper-rs

## Instuctions on starting the prototype

- Get an AMI up and running. Use the Arch Linux AMIs found [here](https://www.uplinklabs.net/projects/arch-linux-on-ec2/) for convenience.
- Collect the IP to a file with format similar to `randpiper-rs/scripts/ip_file`.
- Run `scripts/aws/do_setup.sh` and redirect the input from the IP file. This should look like
```bash
$ bash scripts/aws/do_setup.sh < scripts/ip_file
```
- Run `randpiper-rs/scripts/aws/do_test.sh` and redirect the input from the IP file. This script will run `randpiper-rs/scripts/aws/test.sh` on each machine, so modify `test.sh` beforehand if you want to run different tests.

Inside `test.sh` is a relatively short script:
```bash
killall -9 node-bft
timeout 600 ./randpiper-rs/target/release/node-bft -c ./randpiper-rs/test/d100-n32/nodes-$1.dat -d 280 -i ./randpiper-rs/ips_file > output.log
```

- The timeout duration (`600`) dictates how much time (seconds) is spent running the test, i.e. how many loops will be run.
- The data file (`./randpiper-rs/test/d100-n32/nodes-$1.dat`) is the one the node uses for configuration.
- The delta (`280`) is the performance parameter to be minimized through try-and-error.
- The script `do_test.sh` will download the log file from each node after the test has finished. We want to check the log file to ensure that the beacon is the same across all nodes and no irregularities (e.g. desync) are observed.
