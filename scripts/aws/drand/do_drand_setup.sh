# Do the setup on the AWS Server

FILE="${1:-/dev/stdin}"
PVT_IP_FILE="scripts/aws/pvt_ips.log"
IPS_FILE=${2:-"scripts/aws/ips_file.log"}
CLI_IPS_FILE=${3:-"scripts/aws/cli_ips.log"}
IPS=()

while IFS= read -r line; do
  IPS+=($line)
done < $FILE

for ip in "${IPS[@]}"
do
    ssh -oStrictHostKeyChecking=accept-new -t arch@$ip 'bash -ls' < scripts/aws/drand/drand_setup.sh &
done

wait