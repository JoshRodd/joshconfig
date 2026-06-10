#!/usr/bin/env zsh
# update-web.sh version: 0.1.0
# Push boot.sh and boot.html to the rodd.us webserver.
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
TARGET="jerodd@direct.rodd.us"
WEBROOT="rodd.us"

scp "$DIR/boot.sh" "$DIR/boot.html" "${TARGET}:${WEBROOT}/"
echo "Updated boot.sh and boot.html on rodd.us"
