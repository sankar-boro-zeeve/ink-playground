#!/bin/bash

versions=("v4_2_1" "v4_2_0" "v4_1_0" "v4_0_0")
versions_num=(4.2.1 4.2.0 4.1.0 4.0.0)

COUNT=0

for version in ${versions[*]}
do
	cargo run --package crate-extractor -- create \
			-m $HOME/ink_compilers/contract_$version/Cargo.lock \
			-o packages/_generated/change/src/change_$version.json
done
