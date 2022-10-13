#!/bin/sh
# On localhost, there is a database where shapefiles have been imported using shp2pgsql.
# This exports the all the GIS tables from localhost and converts them into a format suitable for import into Heroku.
DB=mngis1
NOW=`date +"%Y%m%d"`
SQL=$DB-all-$NOW.sql
pg_dump -Fc --no-acl --no-owner --format plain --clean -h localhost -p 5433 -U cperez mngis1 > $SQL
# Modify
NEW_SCHEMA=p6t_state_mn
# Database mngis1 is on the public schema. Instead, put our custom tables on a different schema.
sed -i.orig -E \
    -e "/(geom |spatial_ref_sys)/! s/ public\./ $NEW_SCHEMA\./g" \
    -e "s/(nextval|setval)\('public./\1('$NEW_SCHEMA./g" \
    -e 's/DROP EXTENSION postgis;/-- DROP EXTENSION postgis;/g' \
    $SQL
echo Wrote $SQL
# Then copy in:
# psql populist-platform-dev < mngis1-all-20221009.sql
