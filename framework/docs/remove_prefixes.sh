#!/bin/bash

# Check if the directory is specified as an argument
if [ -z "$1" ]; then
  echo "Please provide the directory path."
  exit 1
fi

# Set the target directory
TARGET_DIR=$1

echo "Processing directories..."

# Remove prefixes from directories first
find "$TARGET_DIR" -depth -type d | while read -r DIR; do
  # Extract the base name of the directory
  BASENAME=$(basename "$DIR")
  # Print the directory being checked
  echo "Checking directory: $DIR"
  # Remove the numeric prefix
  NEWNAME=$(echo "$BASENAME" | sed -E 's/^[0-9]+_//')
  # Rename the directory if necessary
  if [ "$BASENAME" != "$NEWNAME" ]; then
    NEWDIR=$(dirname "$DIR")/"$NEWNAME"
    echo "Renaming directory: $DIR -> $NEWDIR"
    mv "$DIR" "$NEWDIR"
  fi
done

echo "Processing files..."

# Remove prefixes from files
find "$TARGET_DIR" -type f | while read -r FILE; do
  # Extract the base name of the file
  BASENAME=$(basename "$FILE")
  # Print the file being checked
  echo "Checking file: $FILE"
  # Remove the numeric prefix
  NEWNAME=$(echo "$BASENAME" | sed -E 's/^[0-9]+_//')
  # Rename the file if necessary
  if [ "$BASENAME" != "$NEWNAME" ]; then
    NEWFILE=$(dirname "$FILE")/"$NEWNAME"
    echo "Renaming file: $FILE -> $NEWFILE"
    mv "$FILE" "$NEWFILE"
  fi
done

echo "Prefix removal completed."
