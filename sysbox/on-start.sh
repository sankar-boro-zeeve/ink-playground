#!/bin/sh

# dockerd start
dockerd > /var/log/dockerd.log 2>&1 &
sleep 2

# pull inner images
docker pull sankarboro/ink-compiler:latest
docker tag sankarboro/ink-compiler ink-compiler

# start backend server
/app/target/release/backend --port 4000 --host 0.0.0.0 --frontend_folder /app/packages/playground/dist

