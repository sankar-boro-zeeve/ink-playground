#!/bin/bash

# local
run_local() {
    echo "running local..."
    jq --version
    ink_releases=$( jq -r '.[] | .ink_version' ./config/ink-data/stable_ink_releases.json; )
    echo $ink_releases
}

run_local

COUNT=0
declare -i COUNT

build() {
    cargo update -p ink --precise $1
    cargo update -p ink_env --precise $1
    cargo update -p ink_storage --precise $1
    cargo update -p ink_ir --precise $1
    cargo update -p ink_prelude --precise $1
    cargo update -p ink_allocator --precise $1
    cargo update -p ink_primitives --precise $1
    cargo update -p ink_engine --precise $1
    cargo update -p ink_codegen --precise $1
    cargo update -p ink_macro --precise $1
    cargo update -p ink_storage_traits --precise $1
    cargo update -p ink_metadata --precise $1
}

thisdir="/home/sankar/ink-data"
rm -rf -p $thisdir
mkdir -p $thisdir

for ink_version in $ink_releases
do
    v_string=$(echo "$ink_version" | sed 's/\./_/g')
    cd $thisdir
    cargo contract new contract_$v_string
    cd contract_$v_string
    cargo update -p ink --precise ${ink_version:0:5}
    for i in 0 1 2 3 4 5
    do
        build ${ink_version:0:5}
    done
    cargo contract build
    ls -la
    sleep 10
    cd ..
    COUNT+=1
done