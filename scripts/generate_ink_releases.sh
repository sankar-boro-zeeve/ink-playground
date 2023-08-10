#!/bin/bash

work_dir_path=$(pwd)
ink_data_path=$work_dir_path/config/ink-data
mkdir -p $ink_data_path

ink_release_data=$ink_data_path/ink_release_data.json
ink_release_tags=$ink_data_path/ink_release_tags.json
ink_releases=$ink_data_path/ink_releases.json
stable_ink_releases=$ink_data_path/stable_ink_releases.json


rm -rf $config_path/ink*

curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/paritytech/ink/releases -o $ink_release_data

jq '[ .[] | { tag_name }]' $ink_release_data > $ink_release_tags
jq 'map(select(.tag_name | length < 7))' $ink_release_tags > $ink_releases
jq '.[:5]' $ink_releases > $stable_ink_releases
