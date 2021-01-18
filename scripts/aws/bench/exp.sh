# Benchmark the crypto operations
if [ $# -ne 1 ]; then
    echo "Please specify a run prefix"
    echo "Usage: $0 <Run prefix>"
    exit 1
fi

if [ -e $1 ]; then
    echo "Run directory [$1] already exists"
    exit 0
fi

mkdir -p "$1"

SERVER="`head -1 scripts/aws/aws_ips.log`"
echo "Talking to server: $SERVER"

ssh arch@$SERVER 'source $HOME/.cargo/env; cd randpiper-rs; cargo bench -p crypto' &> $1/bench.log