#!/bin/bash
set -e

# Path to binaries (assuming run from repo root)
BD_GO="$(pwd)/bd-go"
BD_RUST="$(pwd)/bd-rust"

if [ ! -f "$BD_GO" ]; then
    echo "Error: bd-go not found at $BD_GO"
    exit 1
fi

if [ ! -f "$BD_RUST" ]; then
    echo "Error: bd-rust not found at $BD_RUST"
    exit 1
fi

TEST_DIR=$(mktemp -d -t beads-compat-XXXXXX)
echo "Running compatibility tests in $TEST_DIR"

cleanup() {
    # echo "Cleaning up..."
    # rm -rf "$TEST_DIR"
    echo "Test dir left at $TEST_DIR for inspection"
}
trap cleanup EXIT

cd "$TEST_DIR"

# 1. Initialize with Go
echo "--- Step 1: Onboard with Go ---"
git init
# Configure git user for beads
git config user.name "TestUser"
git config user.email "test@example.com"

# bd init creates the database
"$BD_GO" init --prefix bd

if [ ! -d ".beads" ]; then
    echo "Error: .beads directory not created"
    exit 1
fi

# 2. Create issue with Go
echo "--- Step 2: Create issue with Go ---"
"$BD_GO" create "First Issue" --description "Created by Go" --priority 1
# Get ID.
# bd-go list outputs table.
# Let's use grep/awk to get the ID.
# Format: ID | Status | Priority | Title
ISSUE_ID=$("$BD_GO" list | grep "First Issue" | awk '{print $1}')
echo "Created Issue ID: $ISSUE_ID"

if [ -z "$ISSUE_ID" ]; then
    echo "Error: Could not find created issue"
    exit 1
fi

# 3. Read with Rust
echo "--- Step 3: Read with Rust ---"
"$BD_RUST" show "$ISSUE_ID"
# Verify content
"$BD_RUST" show "$ISSUE_ID" | grep "Created by Go" > /dev/null
if [ $? -ne 0 ]; then
    echo "Error: Rust failed to read description created by Go"
    exit 1
fi

# 4. Update with Rust
echo "--- Step 4: Update with Rust ---"
"$BD_RUST" update "$ISSUE_ID" --description "Updated by Rust" --status "in_progress"

# 5. Verify with Go
echo "--- Step 5: Verify update with Go ---"
"$BD_GO" show "$ISSUE_ID" | grep "Updated by Rust" > /dev/null
if [ $? -ne 0 ]; then
    echo "Error: Go failed to read description updated by Rust"
    exit 1
fi

STATUS_CHECK=$("$BD_GO" show "$ISSUE_ID" | grep "Status:" | grep "in_progress")
if [ -z "$STATUS_CHECK" ]; then
    echo "Error: Go failed to see status update from Rust"
    exit 1
fi

# 6. Export with Rust (Sync)
echo "--- Step 6: Export with Rust (Sync) ---"
# sync requires remote? No, just git.
# But sync does pull --rebase.
# We have a fresh git repo, no remote.
# Sync might fail on pull if no remote?
# beads-core sync logic:
# if git_pull_rebase fails, it errors?
# Let's see.
# It tries to find conflicts.
# If no remote, pull might say "There is no tracking information".
# We should probably commit first locally so we have a HEAD.
git add .
git commit -m "Initial commit"

# Now run sync.
"$BD_RUST" sync

# Check JSONL
if [ ! -f ".beads/issues.jsonl" ]; then
    echo "Error: JSONL file not created by Rust sync"
    exit 1
fi

grep "Updated by Rust" .beads/issues.jsonl > /dev/null
if [ $? -ne 0 ]; then
    echo "Error: JSONL does not contain updated description"
    exit 1
fi

# 7. Import with Go (Sync)
# To test import, we should modify the JSONL or create a new issue in JSONL?
# Or we can just ensure Go sync doesn't explode.
echo "--- Step 7: Import with Go (Sync) ---"
"$BD_GO" sync

# Check if DB is still intact
"$BD_GO" show "$ISSUE_ID" | grep "Updated by Rust" > /dev/null
if [ $? -ne 0 ]; then
    echo "Error: Go failed to read issue after sync"
    exit 1
fi

echo "SUCCESS: Cross-compatibility verification passed!"
