for file in $1/*.txt ; do
    file=`basename $file .txt`
    echo $file
    grep "time_" $1/$file.txt | cut -d= -f6 | sort -n > $1/$file-sorted.txt
done