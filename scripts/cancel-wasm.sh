#!/bin/bash

# Set the project slug and CircleCI API token
PROJECT_SLUG="gh/AbstractSDK/abstract"
CIRCLECI_TOKEN=$CCI_TOKEN

# Get the current branch name from environment variables

# Fetch running builds for the project
RUNNING_BUILDS=$(curl -s -H "Circle-Token: $CIRCLECI_TOKEN" \
                 "https://circleci.com/api/v2/project/$PROJECT_SLUG/pipeline?branch=main&status=running")

echo $RUNNING_BUILDS

# Iterate over each workflow and get their jobs
for WORKFLOW_ID in $(echo $WORKFLOWS | jq -r '.items[] | select(.name == "build-and-commit") | .id'); do
    JOBS=$(curl -s -H "Circle-Token: $CIRCLECI_TOKEN" \
                "https://circleci.com/api/v2/workflow/$WORKFLOW_ID/job")

    echo $JOBS

    # Iterate over each job and cancel if it is running
    for JOB_ID in $(echo $JOBS | jq -r '.items[] | select(.status == "running") | .id'); do
        curl -X POST -H "Circle-Token: $CIRCLECI_TOKEN" \
             "https://circleci.com/api/v2/project/$PROJECT_SLUG/job/$JOB_ID/cancel"
    done
done