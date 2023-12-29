#!/bin/bash

# Check if a tag name is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <tag-name>"
    exit 1
fi

TAG_NAME=$1

# Function to check OS type for sed command compatibility
function sed_in_place() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "$@"
    else
        sed -i "$@"
    fi
}

# Function to restore the original state in case of an error
function restore_state {
    git reset --hard HEAD
    git checkout .
    git tag -d $TAG_NAME
    echo "Error occurred. Restored the original state."
}

# Stashing any uncommitted changes
git stash

# Removing **/Cargo.lock from .gitignore
sed_in_place '/\*\*\/Cargo.lock/d' .gitignore

# Adding Cargo.lock files and committing
git add $(find . -name Cargo.lock) .gitignore
git commit -m "Add Cargo.lock for tag $TAG_NAME"

# Tagging the commit
git tag -a $TAG_NAME -m "Version $TAG_NAME with Cargo.lock"

# Pushing the tag
if git push origin $TAG_NAME; then
    echo "Tag $TAG_NAME pushed successfully."
else
    restore_state
    exit 1
fi

# Reverting changes
git reset --hard HEAD~1
git checkout .gitignore

# Re-applying stashed changes
git stash pop

echo "Completed. Tag $TAG_NAME contains Cargo.lock."
