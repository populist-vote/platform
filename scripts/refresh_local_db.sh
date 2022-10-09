#!/bin/bash

# usage: ./refresh_local_db.sh <app> [--nocapture]
set -e

# DB configuration
DATABASE="populist-platform-dev"
OWNER=`whoami`
# If you want to use an explicit PG version, host, and port for the PG commands, export PGCLUSTER before running this script:
# export PGCLUSTER="12//var/run/postgresql:5433"

if [[ $1 ]]; then
    APP=$1
else
    echo "An app name on Heroku is required (populist-api-staging, populist-api-production)"
    exit 0
fi

# Optional: don't run heroku:pgbackups:capture, just use latest backup by setting --nocapture
CAPTURE=${2:-capture}

echo "Checking if logged into Heroku..."
if ! heroku auth:whoami ; then
    echo "Please login to Heroku first: 'heroku login -i'"
    exit 1
fi

function do_cleanup() {
    dropdb $DATABASE
    echo "Cancelled local database dump"
    echo
}

trap do_cleanup INT

echo "This will dump from Heroku app '$APP' to your local database '$DATABASE'."
read -p "Would you like to proceed? (y/n):" -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    if ! heroku info --app=$APP 1>/dev/null ; then
        echo "App '$APP' doesn't seem to exist. Typo?"
        exit 1
    fi

    echo "Dropping local database $DATABASE"
    dropdb --if-exists $DATABASE

    echo "Creating local database $DATABASE"
    sudo -E -u postgres createdb -T template0 -O $OWNER $DATABASE

    # Need superuser privileges to create extensions
    sudo -E -u postgres psql -c "alter role $OWNER superuser;"

    # To get a recent dump, run heroku pg:backups:capture first.
    echo "Retrieving database dump from $APP"
    if [ "$CAPTURE" != "--nocapture" ]; then
        heroku pg:backups:capture --app=$APP
    fi
    echo "Restoring locally..."
    # Necessary to set gen_random_uuid to public.gen_random_uuid otherwise you get 'ERROR:  function gen_random_uuid() does not exist'
    heroku pg:backups:url --app=$APP | xargs curl | pg_restore --no-owner --no-acl -f - | sed 's/"gen_random_uuid"()/"public"."gen_random_uuid"()/g' | psql $DATABASE

    # No longer need superuser privileges
    sudo -E -u postgres psql -c "alter role $OWNER nosuperuser;"

    echo "Data from '$APP' has been dumped and restored to '$DATABASE'"
    echo "Done!"
    echo
else
    echo "Cancelled local database dump"
    echo
fi
