#!/bin/sh
# test-loader.sh - Test the load-paths.sh script
# This script tests the loader in both bash and zsh

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LOADER_SCRIPT="$SCRIPT_DIR/../scripts/load-paths.sh"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

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

# Create a temporary home directory
TEMP_HOME=$(mktemp -d)
trap "rm -rf $TEMP_HOME" EXIT

# Create test .paths.d
mkdir -p "$TEMP_HOME/.paths.d"
mkdir -p "$TEMP_HOME/.manpaths.d"

# Test 1: Basic path loading
echo "Test 1: Basic path loading"
cat > "$TEMP_HOME/.paths.d/10-usr-local-bin" << 'EOF'
/usr/local/bin # Local binaries
EOF

cat > "$TEMP_HOME/.paths.d/20-opt-bin" << 'EOF'
/opt/bin # Optional packages
EOF

HOME="$TEMP_HOME" PATH="/usr/bin:/bin" sh -c ". $LOADER_SCRIPT && echo \$PATH" | grep -q "^/opt/bin:/usr/local/bin:/usr/bin:/bin$"
if [ $? -eq 0 ]; then
    pass "Paths loaded in correct order"
else
    fail "Paths not loaded correctly"
fi

# Test 2: Deduplication
echo "Test 2: Deduplication"
rm -rf "$TEMP_HOME/.paths.d"/*
cat > "$TEMP_HOME/.paths.d/10-first" << 'EOF'
/usr/local/bin # First
EOF

cat > "$TEMP_HOME/.paths.d/20-second" << 'EOF'
/usr/local/bin # Second (duplicate)
EOF

result=$(HOME="$TEMP_HOME" PATH="/usr/bin:/bin" sh -c ". $LOADER_SCRIPT && echo \$PATH")
if echo "$result" | grep -q "^/usr/local/bin:/usr/bin:/bin$"; then
    pass "Duplicates removed, first occurrence kept"
else
    fail "Deduplication failed: $result"
fi

# Test 3: $HOME expansion
echo "Test 3: \$HOME expansion"
rm -rf "$TEMP_HOME/.paths.d"/*
cat > "$TEMP_HOME/.paths.d/10-home-bin" << 'EOF'
$HOME/.local/bin # User local
EOF

result=$(HOME="$TEMP_HOME" PATH="/usr/bin:/bin" sh -c ". $LOADER_SCRIPT && echo \$PATH")
if echo "$result" | grep -q "$TEMP_HOME/.local/bin"; then
    pass "\$HOME expanded correctly"
else
    fail "\$HOME expansion failed: $result"
fi

# Test 4: Tilde expansion
echo "Test 4: Tilde expansion"
rm -rf "$TEMP_HOME/.paths.d"/*
cat > "$TEMP_HOME/.paths.d/10-home-bin" << 'EOF'
~/bin # User bin
EOF

result=$(HOME="$TEMP_HOME" PATH="/usr/bin:/bin" sh -c ". $LOADER_SCRIPT && echo \$PATH")
if echo "$result" | grep -q "$TEMP_HOME/bin"; then
    pass "Tilde expanded correctly"
else
    fail "Tilde expansion failed: $result"
fi

# Test 5: MANPATH loading
echo "Test 5: MANPATH loading"
rm -rf "$TEMP_HOME/.manpaths.d"/*
cat > "$TEMP_HOME/.manpaths.d/10-local-man" << 'EOF'
/usr/local/share/man # Local man pages
EOF

result=$(HOME="$TEMP_HOME" MANPATH="/usr/share/man" sh -c ". $LOADER_SCRIPT && echo \$MANPATH")
if echo "$result" | grep -q "^/usr/local/share/man:/usr/share/man$"; then
    pass "MANPATH loaded correctly"
else
    fail "MANPATH loading failed: $result"
fi

# Test 6: Comments are stripped
echo "Test 6: Comments are stripped"
rm -rf "$TEMP_HOME/.paths.d"/*
cat > "$TEMP_HOME/.paths.d/10-test" << 'EOF'
/usr/local/bin # This is a comment
EOF

result=$(HOME="$TEMP_HOME" PATH="/usr/bin:/bin" sh -c ". $LOADER_SCRIPT && echo \$PATH")
if ! echo "$result" | grep -q "comment"; then
    pass "Comments stripped correctly"
else
    fail "Comments not stripped: $result"
fi

# Test 7: Quoted paths
echo "Test 7: Quoted paths"
rm -rf "$TEMP_HOME/.paths.d"/*
cat > "$TEMP_HOME/.paths.d/10-quoted" << 'EOF'
"/usr/local/bin" # Quoted path
EOF

result=$(HOME="$TEMP_HOME" PATH="/usr/bin:/bin" sh -c ". $LOADER_SCRIPT && echo \$PATH")
if echo "$result" | grep -q "^/usr/local/bin:/usr/bin:/bin$"; then
    pass "Quoted paths handled correctly"
else
    fail "Quoted paths failed: $result"
fi

# Test 8: Empty directory
echo "Test 8: Empty directory"
rm -rf "$TEMP_HOME/.paths.d"/*
result=$(HOME="$TEMP_HOME" PATH="/usr/bin:/bin" sh -c ". $LOADER_SCRIPT && echo \$PATH")
if [ "$result" = "/usr/bin:/bin" ]; then
    pass "Empty directory handled correctly"
else
    fail "Empty directory failed: $result"
fi

# Test 9: Missing directory
echo "Test 9: Missing directory"
rm -rf "$TEMP_HOME/.paths.d"
result=$(HOME="$TEMP_HOME" PATH="/usr/bin:/bin" sh -c ". $LOADER_SCRIPT && echo \$PATH")
if [ "$result" = "/usr/bin:/bin" ]; then
    pass "Missing directory handled correctly"
else
    fail "Missing directory failed: $result"
fi

# Recreate for remaining tests
mkdir -p "$TEMP_HOME/.paths.d"

# Test 10: Multiple paths in existing PATH
echo "Test 10: Preserve existing PATH order"
rm -rf "$TEMP_HOME/.paths.d"/*
cat > "$TEMP_HOME/.paths.d/10-new" << 'EOF'
/new/path # New path
EOF

result=$(HOME="$TEMP_HOME" PATH="/usr/bin:/bin:/sbin" sh -c ". $LOADER_SCRIPT && echo \$PATH")
if echo "$result" | grep -q "^/new/path:/usr/bin:/bin:/sbin$"; then
    pass "Existing PATH preserved"
else
    fail "Existing PATH not preserved: $result"
fi

echo
echo "==================================="
echo "Test Results: $pass_count/$test_count passed"
if [ $fail_count -gt 0 ]; then
    printf "${RED}$fail_count tests failed${NC}\n"
    exit 1
else
    printf "${GREEN}All tests passed!${NC}\n"
    exit 0
fi
