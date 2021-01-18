# Do the test on the AWS Server

FILE="${1:-/dev/stdin}"
PVT_IP_FILE="scripts/aws/pvt_ips.log"
IPS_FILE=${2:-"scripts/aws/ips_file.log"}
CLI_IPS_FILE=${3:-"scripts/aws/cli_ips.log"}
IPS=()

while IFS= read -r line; do
  IPS+=($line)
done < $FILE

idx=0

for ip in "${IPS[@]}"
do
    ssh -t arch@$ip 'bash -ls --' < scripts/aws/test.sh $idx &
    idx=$(($idx+1))
done

wait

idx=0

for ip in "${IPS[@]}"
do
  scp arch@$ip:./output.log ./$idx.log &
  idx=$(($idx+1))
done

wait
