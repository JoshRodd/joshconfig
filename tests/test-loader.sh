#!/bin/sh
# test-loader.sh - Test zsh and bash loader scripts
# Compatible with bash 3.2.57+ and POSIX sh

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ZSH_LOADER="$SCRIPT_DIR/../scripts/load-paths.zsh"
BASH_LOADER="$SCRIPT_DIR/../scripts/load-paths.bash"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

test_count=0
pass_count=0
fail_count=0

pass() {
    pass_count=$((pass_count + 1))
    test_count=$((test_count + 1))
    printf "${GREEN}✓${NC} %s\n" "$1"
}

fail() {
    fail_count=$((fail_count + 1))
    test_count=$((test_count + 1))
    printf "${RED}✗${NC} %s\n" "$1"
}

reset_dir() {
    _dir="$1"
    mkdir -p "$_dir"
    find "$_dir" -mindepth 1 -delete 2>/dev/null || true
}

run_shell() {
    _shell="$1"
    _loader="$2"
    _home="$3"
    _path_value="$4"
    _manpath_value="$5"
    _expr="$6"

    HOME="$_home" PATH="$_path_value" MANPATH="$_manpath_value" "$_shell" -c ". \"$_loader\" && $_expr"
}

run_suite() {
    _name="$1"
    _shell="$2"
    _loader="$3"

    echo
    echo "Running loader tests for $_name"
    echo "==================================="

    TEMP_HOME=$(mktemp -d)
    mkdir -p "$TEMP_HOME/.paths.d" "$TEMP_HOME/.manpaths.d"

    # Test 1: Basic path loading
    echo "Test 1: Basic path loading ($_name)"
    reset_dir "$TEMP_HOME/.paths.d"
    cat > "$TEMP_HOME/.paths.d/10-usr-local-bin" << 'EOF'
/usr/local/bin # Local binaries
EOF
    cat > "$TEMP_HOME/.paths.d/20-opt-bin" << 'EOF'
/opt/bin # Optional packages
EOF
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin" "" 'printf "%s" "$PATH"')
    if [ "$result" = "/opt/bin:/usr/local/bin:/usr/bin:/bin" ]; then
        pass "$_name: paths loaded in correct order"
    else
        fail "$_name: paths not loaded correctly: $result"
    fi

    # Test 2: Deduplication
    echo "Test 2: Deduplication ($_name)"
    reset_dir "$TEMP_HOME/.paths.d"
    cat > "$TEMP_HOME/.paths.d/10-first" << 'EOF'
/usr/local/bin # First
EOF
    cat > "$TEMP_HOME/.paths.d/20-second" << 'EOF'
/usr/local/bin # Second duplicate
EOF
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin" "" 'printf "%s" "$PATH"')
    if [ "$result" = "/usr/local/bin:/usr/bin:/bin" ]; then
        pass "$_name: duplicates removed, first occurrence kept"
    else
        fail "$_name: deduplication failed: $result"
    fi

    # Test 3: $HOME expansion
    echo "Test 3: \$HOME expansion ($_name)"
    reset_dir "$TEMP_HOME/.paths.d"
    cat > "$TEMP_HOME/.paths.d/10-home-bin" << 'EOF'
$HOME/.local/bin # User local
EOF
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin" "" 'printf "%s" "$PATH"')
    case "$result" in
        *"$TEMP_HOME/.local/bin"*) pass "$_name: \$HOME expanded correctly" ;;
        *) fail "$_name: \$HOME expansion failed: $result" ;;
    esac

    # Test 4: Tilde expansion
    echo "Test 4: Tilde expansion ($_name)"
    reset_dir "$TEMP_HOME/.paths.d"
    cat > "$TEMP_HOME/.paths.d/10-home-bin" << 'EOF'
~/bin # User bin
EOF
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin" "" 'printf "%s" "$PATH"')
    case "$result" in
        *"$TEMP_HOME/bin"*) pass "$_name: tilde expanded correctly" ;;
        *) fail "$_name: tilde expansion failed: $result" ;;
    esac

    # Test 5: MANPATH loading
    echo "Test 5: MANPATH loading ($_name)"
    reset_dir "$TEMP_HOME/.manpaths.d"
    cat > "$TEMP_HOME/.manpaths.d/10-local-man" << 'EOF'
/usr/local/share/man # Local man pages
EOF
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin" "/usr/share/man" 'printf "%s" "$MANPATH"')
    if [ "$result" = "/usr/local/share/man:/usr/share/man" ]; then
        pass "$_name: MANPATH loaded correctly"
    else
        fail "$_name: MANPATH loading failed: $result"
    fi

    # Test 6: Comments are stripped
    echo "Test 6: Comments are stripped ($_name)"
    reset_dir "$TEMP_HOME/.paths.d"
    cat > "$TEMP_HOME/.paths.d/10-test" << 'EOF'
/usr/local/bin # This is a comment
EOF
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin" "" 'printf "%s" "$PATH"')
    case "$result" in
        *comment*) fail "$_name: comments not stripped: $result" ;;
        *) pass "$_name: comments stripped correctly" ;;
    esac

    # Test 7: Quoted paths
    echo "Test 7: Quoted paths ($_name)"
    reset_dir "$TEMP_HOME/.paths.d"
    cat > "$TEMP_HOME/.paths.d/10-quoted" << 'EOF'
"/usr/local/bin" # Quoted path
EOF
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin" "" 'printf "%s" "$PATH"')
    if [ "$result" = "/usr/local/bin:/usr/bin:/bin" ]; then
        pass "$_name: quoted paths handled correctly"
    else
        fail "$_name: quoted paths failed: $result"
    fi

    # Test 8: Empty directory
    echo "Test 8: Empty directory ($_name)"
    reset_dir "$TEMP_HOME/.paths.d"
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin" "" 'printf "%s" "$PATH"')
    if [ "$result" = "/usr/bin:/bin" ]; then
        pass "$_name: empty directory handled correctly"
    else
        fail "$_name: empty directory failed: $result"
    fi

    # Test 9: Missing directory
    echo "Test 9: Missing directory ($_name)"
    rm -rf "$TEMP_HOME/.paths.d"
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin" "" 'printf "%s" "$PATH"')
    if [ "$result" = "/usr/bin:/bin" ]; then
        pass "$_name: missing directory handled correctly"
    else
        fail "$_name: missing directory failed: $result"
    fi

    mkdir -p "$TEMP_HOME/.paths.d"

    # Test 10: Preserve existing PATH order
    echo "Test 10: Preserve existing PATH order ($_name)"
    reset_dir "$TEMP_HOME/.paths.d"
    cat > "$TEMP_HOME/.paths.d/10-new" << 'EOF'
/new/path # New path
EOF
    result=$(run_shell "$_shell" "$_loader" "$TEMP_HOME" "/usr/bin:/bin:/sbin" "" 'printf "%s" "$PATH"')
    if [ "$result" = "/new/path:/usr/bin:/bin:/sbin" ]; then
        pass "$_name: existing PATH preserved"
    else
        fail "$_name: existing PATH not preserved: $result"
    fi

    rm -rf "$TEMP_HOME"
}

if command -v zsh >/dev/null 2>&1; then
    run_suite "zsh" "zsh" "$ZSH_LOADER"
else
    echo "Skipping zsh tests: zsh not found"
fi

run_suite "bash" "/bin/bash" "$BASH_LOADER"

if command -v bash >/dev/null 2>&1 && [ "$(command -v bash)" != "/bin/bash" ]; then
    run_suite "bash-current" "bash" "$BASH_LOADER"
fi

echo
echo "==================================="
echo "Test Results: $pass_count/$test_count passed"
if [ "$fail_count" -gt 0 ]; then
    printf "${RED}%s tests failed${NC}\n" "$fail_count"
    exit 1
else
    printf "${GREEN}All tests passed!${NC}\n"
    exit 0
fi
