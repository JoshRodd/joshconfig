# Complex .bashrc with various PATH patterns

# Prepend paths
export PATH=/usr/local/bin:/usr/local/sbin:$PATH

# Append paths
export PATH=$PATH:/opt/bin

# With quotes
export PATH="$HOME/.local/bin:$PATH"

# Command substitution
export PATH="$(brew --prefix)/bin:$PATH"

# MANPATH
export MANPATH=/usr/local/man:$MANPATH
export MANPATH=$MANPATH:/opt/man

# Zsh-style arrays (should be ignored in bash parsing)
path=(/new/path $path)

# Multiple paths in one line
PATH=/opt/bin:/usr/local/bin:/usr/bin:$PATH
