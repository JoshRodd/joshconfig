#!/usr/bin/env zsh
# update-web.sh — push boot.sh and boot.html to the rodd.us webserver.
set -euo pipefail

TARGET="jerodd@direct.rodd.us"
WEBROOT="rodd.us"

scp bootstrap/boot.sh bootstrap/boot.html "${TARGET}:${WEBROOT}/"
echo "Updated boot.sh and boot.html on rodd.us"
