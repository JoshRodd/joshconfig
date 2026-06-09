#!/bin/sh
# joshconfig-load-paths.sh v0.1.0 — joshconfig PATH/MANPATH loader shim
# Source this at the END of your .zshrc / .bashrc / .profile
#
# Usage: . "$HOME/.local/bin/joshconfig-load-paths.sh"


# Detect our own path (the directory this shim lives in)
_jc_self=""
_jc_self_dir=""
if [ -n "${BASH_VERSION:-}" ]; then
    _jc_self="${BASH_SOURCE[0]}"
elif [ -n "${ZSH_VERSION:-}" ]; then
    _jc_self="$0"
fi
if [ -n "$_jc_self" ]; then
    _jc_self_dir="${_jc_self%/*}"
fi

# Resolve joshconfig-env binary
if [ -n "${JOSHCONFIG_ENV_BIN:-}" ] && [ -x "$JOSHCONFIG_ENV_BIN" ]; then
    _jc_env_bin="$JOSHCONFIG_ENV_BIN"
elif [ -n "$_jc_self_dir" ] && [ -x "$_jc_self_dir/joshconfig-env" ]; then
    _jc_env_bin="$_jc_self_dir/joshconfig-env"
else
    echo "joshconfig: joshconfig-env not found" >&2
    return 1 2>/dev/null || exit 1
fi

# Run and eval the output (stdout is the shell commands to eval; stderr passes through to user)
_jc_output="$("$_jc_env_bin")"
_jc_rc=$?
if [ $_jc_rc -ne 0 ]; then
    return $_jc_rc 2>/dev/null || exit $_jc_rc
fi
eval "$_jc_output"

# --- Position check ---
# Detect the file that sourced us (self already detected above)
_jc_caller=""
_jc_self_name=""
if [ -n "$_jc_self" ]; then
    _jc_self_name="${_jc_self##*/}"
fi
if [ -n "${BASH_VERSION:-}" ]; then
    _jc_caller="${BASH_SOURCE[1]}"
elif [ -n "${ZSH_VERSION:-}" ]; then
    _jc_trace="${funcfiletrace[1]%:*}"
    [ -n "$_jc_trace" ] && _jc_caller="$_jc_trace"
fi

if [ -n "$_jc_caller" ] && [ -f "$_jc_caller" ]; then
    _jc_total=$(wc -l < "$_jc_caller" | tr -d ' ')
    # Find the last line referencing our own filename in the caller
    if [ -n "$_jc_self_name" ]; then
        _jc_linenum=$(grep -nF "$_jc_self_name" "$_jc_caller" | tail -1 | cut -d: -f1)
    fi
    if [ -n "$_jc_linenum" ] && [ "$_jc_linenum" -lt "$_jc_total" ]; then
        _jc_after=$((_jc_total - _jc_linenum))
        _jc_tail=$(tail -n "$_jc_after" "$_jc_caller" | grep -v '^[[:space:]]*$' | grep -v '^[[:space:]]*#' || true)
        if [ -n "$_jc_tail" ]; then
            echo "joshconfig: WARNING: loader line is not at the end of $_jc_caller" >&2
            echo "joshconfig: Run 'joshconfig doctor' to fix this automatically." >&2
        fi
    fi
fi

# Clean up our temp vars
unset _jc_env_bin _jc_output _jc_rc
unset _jc_caller _jc_self _jc_self_dir _jc_self_name _jc_trace _jc_total _jc_linenum _jc_after _jc_tail
