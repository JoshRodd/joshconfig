#!/bin/sh
# boot.sh version: 0.1.0
# Redirect to the full joshconfig bootstrap on GitHub.
# Pipe this directly into zsh:
#   curl -fsSL https://rodd.us/boot.sh | zsh
exec curl -fsSL https://raw.githubusercontent.com/JoshRodd/joshconfig/master/bootstrap/bootstrap-mac.sh | zsh
