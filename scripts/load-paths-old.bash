#!/usr/bin/env bash
# load-paths.bash - Load PATH and MANPATH from ~/.paths.d and ~/.manpaths.d
# Bash-specific loader script (compatible with bash 3.2.57+)
#
# Usage: Source this file in your .bashrc
#   . "$HOME/.local/bin/load-paths.bash"

# Function to add a path to a variable if not already present
# Uses case statement for POSIX compatibility (no associative arrays)
_add_path_if_new() {
    _path_var="$1"
    _new_path="$2"
    _current_paths="$3"

    # Check if path is already in the list
    case ":${_current_paths}:" in
        *":${_new_path}:"*)
            # Already present, return unchanged
            printf '%s' "$_current_paths"
            ;;
        *)
            # Not present, add it
            if [ -z "$_current_paths" ]; then
                printf '%s' "$_new_path"
            else
                printf '%s:%s' "$_new_path" "$_current_paths"
            fi
            ;;
    esac
}

# Function to append a path to a variable if not already present
# Used for deduplication to maintain order
_append_path_if_new() {
    _path_var="$1"
    _new_path="$2"
    _current_paths="$3"

    # Check if path is already in the list
    case ":${_current_paths}:" in
        *":${_new_path}:"*)
            # Already present, return unchanged
            printf '%s' "$_current_paths"
            ;;
        *)
            # Not present, append it
            if [ -z "$_current_paths" ]; then
                printf '%s' "$_new_path"
            else
                printf '%s:%s' "$_current_paths" "$_new_path"
            fi
            ;;
    esac
}

# Function to expand $HOME in a path
_expand_home() {
    _path="$1"
    case "$_path" in
        *'$HOME'*|*'~'*)
            # Replace $HOME and ~ with actual home directory
            printf '%s' "$_path" | sed "s|\\\$HOME|$HOME|g; s|^~|$HOME|"
            ;;
        *)
            printf '%s' "$_path"
            ;;
    esac
}

# Function to load paths from a .d directory
_load_paths_from_dir() {
    _dir="$1"
    _var_name="$2"

    # Get current value of the variable
    eval "_current_value=\"\${$_var_name}\""

    if [ ! -d "$_dir" ]; then
        return
    fi

    # Check if directory has any files
    if [ -z "$(ls -A "$_dir" 2>/dev/null)" ]; then
        return
    fi

    # Build new path by reading files in sorted order
    _new_paths=""
    for _file in "$_dir"/*; do
        [ -f "$_file" ] || continue

        # Read first non-comment line
        _line=""
        while IFS= read -r _line || [ -n "$_line" ]; do
            # Skip empty lines and comment-only lines
            case "$_line" in
                ''|\#*)
                    continue
                    ;;
            esac

            # Strip inline comments (but not inside quotes)
            # Simple approach: remove everything after # that's preceded by whitespace
            _line=$(printf '%s' "$_line" | sed 's/[[:space:]]*#.*$//')

            # Strip quotes
            _line=$(printf '%s' "$_line" | sed 's/^["'\'']\(.*\)["'\'']$/\1/')

            # Expand $HOME
            _line=$(_expand_home "$_line")

            # Add to new paths if not already present
            _new_paths=$(_add_path_if_new "$_var_name" "$_line" "$_new_paths")

            # Only process first valid line per file
            break
        done < "$_file"
    done

    # Combine: new paths + existing paths, then deduplicate
    if [ -n "$_new_paths" ]; then
        if [ -n "$_current_value" ]; then
            _combined="${_new_paths}:${_current_value}"
        else
            _combined="$_new_paths"
        fi

        # Deduplicate, keeping first occurrence (append to maintain order)
        # Bash 3.2 supports indexed arrays; split only on ':' so spaces in paths survive.
        _deduped=""
        _old_ifs="$IFS"
        IFS=':' read -r -a _path_array <<< "$_combined"
        IFS="$_old_ifs"

        for _path in "${_path_array[@]}"; do
            [ -z "$_path" ] && continue
            _deduped=$(_append_path_if_new "$_var_name" "$_path" "$_deduped")
        done

        # Set the variable
        eval "$_var_name=\"\$_deduped\""
        export "$_var_name"
    fi
}

# Main: load both PATH and MANPATH
_load_paths_from_dir "$HOME/.paths.d" PATH
_load_paths_from_dir "$HOME/.manpaths.d" MANPATH
