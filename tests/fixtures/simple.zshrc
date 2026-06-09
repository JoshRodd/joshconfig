# Simple .zshrc with basic PATH modifications

export PATH=/usr/local/bin:$PATH
export PATH=$PATH:/opt/homebrew/bin
export MANPATH=/usr/local/share/man:$MANPATH

# Some other config
alias ll='ls -la'
