#!/bin/bash
export PATH=$PATH:/usr/local/bin

#
# contracts pre-push hook, used to perform static analysis checks on changed files.
#
# Install the hook with the --install option.
#

project_toplevel=$(git rev-parse --show-toplevel)
git_directory=$(git rev-parse --git-dir)

install_hook() {
  mkdir -p "$git_directory/hooks"
  ln -sfv "$project_toplevel/hooks/pre-push.sh" "$git_directory/hooks/pre-push"
}

if [ "$1" = "--install" ]; then
  if [ -f "$git_directory/hooks/pre-push" ]; then
    read -r -p "There's an existing pre-push hook. Do you want to overwrite it? [y/N] " response
    case "$response" in
    [yY][eE][sS] | [yY])
      install_hook
      ;;
    *)
      printf "Skipping hook installation :("
      exit $?
      ;;
    esac
  else
    install_hook
  fi
  exit $?
fi

# cargo fmt checks
format_check() {
  printf "Starting file formatting check...\n"

  cd $project_toplevel || exit;
  cargo fmt;
  cargo workspaces exec --no-bail cargo schema >/dev/null;
  sleep 3; # Give git time to find changed files.
  not_staged_file=$(git diff --name-only)
    if [ "$not_staged_file" != "" ]; then # it means the file changed and it's not staged, i.e. rustfmt did the job.
      git add .
      git commit -m "formatting and schema gen"
    fi
}

# clippy checks
lint_check() {
  printf "Starting clippy check...\n"
  cargo clippy --quiet -- -D warnings
  clippy_exit_code=$?
  if [ $clippy_exit_code -ne 0 ]; then
    printf "\nclippy found some issues. Fix them manually and try again :)"
    exit 1
  fi
}

# # schema checks
# schema_check() {
#   printf "Starting file formatting check...\n"

#   has_formatting_issues=0
#   first_file=1
#   rust_staged_files=$(git diff --name-only --staged -- '*.rs')

#   # check for issues
#   for file in $rust_staged_files; do
#     format_check_result=$(rustfmt --check $file)
#     if [ "$format_check_result" != "" ]; then
#       if [ $first_file -eq 0 ]; then
#         printf "\n"
#       fi
#       printf "$file"
#       has_formatting_issues=1
#       first_file=0
#     fi
#   done

#   if [ $has_formatting_issues -ne 0 ]; then # there are formatting issues
#     printf "\nFormatting issues were found in files listed above. Trying to format them for you...\n"
#     exit_code=0

#     for file in $rust_staged_files; do
#       rustfmt $file
#       format_exit_code=$?

#       if [ $format_exit_code -ne 0 ]; then
#         # rustfmt couldn't format the current file
#         exit_code=1
#       else
#         not_staged_file=$(git diff --name-only -- $file)

#         if [ "$not_staged_file" != "" ]; then # it means the file changed and it's not staged, i.e. rustfmt did the job.
#           git add $not_staged_file
#         fi
#       fi
#     done

#     if [ $exit_code -ne 0 ]; then
#       printf "rustfmt failed to format some files. Please review, fix them and stage them manually."
#       exit 1
#     fi

#   fi
# }
lint_check
format_check
