#!/bin/bash

ink_releases=$( jq -r '.[] | .ink_version' ./config/ink-data/stable_ink_releases.json; )
echo $ink_releases
for version in ${ink_releases[*]}
do
	echo $version
	thisversion=$(echo "$version" | sed 's/\./_/g')
	echo $thisversion 
	cargo run --package crate-extractor --locked -- create \
			-m $HOME/ink-data/contract_$thisversion/Cargo.lock \
			-o packages/_generated/change/src/change_$thisversion.json
done
