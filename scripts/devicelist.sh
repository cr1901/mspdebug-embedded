#!/bin/sh

if [ $# -lt 1 ]; then
    echo "Usage: $0 [mspdebug_root] [msp430-gcc-support-files_root]"
    exit 1
fi

grep -o \".*\" $1/drivers/devicelist.c | \
    tr -d "\"" | \
    sed -e 's/devicelist.h//' | \
    sort | \
    uniq > mspdebug.txt
(cd $2/include; grep INFOMEM -R .) | \
    sed -e 's%./%%' -e 's/.ld//' | \
    cut -d':' -f1,3 | \
    tr [:lower:] [:upper:] | \
    sed -e 's%/\*%,%' -e 's%:%,%' -e s'%*/%%' | \
    sort > mspheaders.txt

python3 mkphf.py mspdebug.txt mspheaders.txt
