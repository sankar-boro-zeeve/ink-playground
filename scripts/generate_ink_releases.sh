#!/bin/bash

work_dir_path=$(pwd)
config_path=$work_dir_path/config

ink_release_data=$config_path/ink_release_data.json
ink_release_tags=$config_path/ink_release_tags.json
ink_releases=$config_path/ink_releases.json

rm -rf $config_path/ink*

curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/paritytech/ink/releases -o $ink_release_data

jq '[ .[] | { tag_name }]' $ink_release_data > $ink_release_tags
jq 'map(select(.tag_name | length < 7))' $ink_release_tags > $ink_releases