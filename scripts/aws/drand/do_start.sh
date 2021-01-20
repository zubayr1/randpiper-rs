# Do the setup on the AWS Server

FILE="${1:-/dev/stdin}"
IPS=()

while IFS= read -r line; do
  IPS+=($line)
done < $FILE

N=${#IPS[@]}
Nm1=$(( $N - 1 ))

LeaderAddr=${IPS[$Nm1]}

for((i=0;i<$N;i++))
do
    ip=${IPS[$i]}
    if [ $i == $Nm1 ]; then
        ssh -t arch@$ip 'timeout 300 bash -ls --' < scripts/aws/drand/start.sh "$LeaderAddr:8080" $N "leader" &
    else
        ssh -t arch@$ip 'timeout 300 bash -ls --' < scripts/aws/drand/start.sh "$LeaderAddr:8080" $N &
    fi
done

wait