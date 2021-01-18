# A script that takes the bench.log produced in the experiment and produces a file that can be consumed by a python file to produce a CSV file

grep '.*\/.*time:' $1/bench.log > $1/bench-cleaned.log

KEYS=("bi_sh_gen" "bi_sh_vrfy" "evss_sh_gen" "evss_sh_vrfy" "evss_sh_recon" "pvss_sh_gen" "pvss_sh_vrfy" "pvss_sh_recon")

for k in ${KEYS[@]}; do 
    for x in `grep "$k" $1/bench-cleaned.log | cut -d"/" -f2 | cut -d" " -f1`; do
        y=`grep "$k" $1/bench-cleaned.log | grep "$k/$x " | cut -d"[" -f2 | cut -d" " -f3,4`
        echo "$k,$x,$y"
    done
done > $1/bench-processed.log

python scripts/aws/bench/parse-exp.py $1/bench-processed.log $1/bench-processed.json