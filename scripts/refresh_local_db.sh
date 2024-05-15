#!/bin/bash

# Check if the app name argument is provided
if [ -z "$1" ]; then
  echo "Please provide the Heroku app name as an argument."
  exit 1
fi

# Drop the local database if it exists
dropdb --if-exists populist-platform-dev

# Capture a backup from Heroku
heroku pg:backups:capture --app $1
heroku pg:backups:download --app $1

createdb -h localhost -U postgres populist-platform-dev

# Restore the backup locally
pg_restore --verbose --clean --no-acl --no-owner -h localhost -U postgres -d populist-platform-dev latest.dump

# Cleanup
rm latest.dump