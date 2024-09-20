#!/bin/bash

curl -X POST https://content.dropboxapi.com/2/files/upload \
    --header "Authorization: Bearer $ACCESS_TOKEN" \
    --header "Dropbox-API-Arg: {\"path\": \"/aarch64/xode-node\", \"mode\": \"overwrite\", \"strict_conflict\": false}" \
    --header "Content-Type: application/octet-stream" \
    --data-binary @xode-node
