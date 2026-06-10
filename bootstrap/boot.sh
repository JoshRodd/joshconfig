#!/bin/sh
# boot.sh — one-liner bootstrap for a new Mac.
# Pipe this directly into zsh:
#   curl -fsSL https://rodd.us/boot.sh | zsh
exec curl -fsSL https://raw.githubusercontent.com/JoshRodd/joshconfig/master/bootstrap/bootstrap-mac.sh | zsh
