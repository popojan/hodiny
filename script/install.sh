#!/bin/sh

cp -f target/release/hodiny .
echo "Please add this line to your cron configuration" 1>&2
echo "*/15 * * * * `whoami`  cd `pwd` && ./hodiny"
