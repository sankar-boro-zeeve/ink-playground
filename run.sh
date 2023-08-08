echo $1 $2;

cd /home/sankar/zeeve/test-contracts/contracts/$1
echo "building contract for project: $1"

cargo contract build