#!/bin/bash

work_dir_path=$(pwd)
ink_data_path=$work_dir_path/config/ink-data
mkdir -p $ink_data_path

ink_release_data=$ink_data_path/ink_release_data.json
ink_release_versions=$ink_data_path/ink_release_versions.json
stable_ink_releases=$ink_data_path/stable_ink_releases.json


rm -rf $config_path/ink*

curl https://crates.io/api/v1/crates/ink -o $ink_release_data

jq '[ .versions[] | { num } | { ink_version: .num }] | map(select(.ink_version | length < 7)) | .[0:-1]' $ink_release_data > $stable_ink_releases
