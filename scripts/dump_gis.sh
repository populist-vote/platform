#!/bin/sh
# pg_dump -Fc --no-acl --no-owner --format plain -h localhost -p 5433 -U cperez --table public.school_district_boundaries mngis1 > mngis1-schools-20221004.sql
# sed -i.orig 's/public.geometry/heroku_ext.geometry/g' mngis1-schools-20221004.sql
pg_dump -Fc --no-acl --no-owner --format plain --clean -h localhost -p 5433 -U cperez mngis1 > mngis1-all-20221009.sql
# Database mngis1 is on the public schema. Instead, put our custom tables on schema p6t_state_mn.
sed -i.orig -E \
    -e '/(geom |spatial_ref_sys)/! s/ public\./ p6t_state_mn\./g' \
    -e "s/nextval\('public./nextval('p6t_state_mn./g" \
    -e "s/setval\('public./setval('p6t_state_mn./g" \
    -e 's/DROP EXTENSION postgis;/-- DROP EXTENSION postgis;/g' \
    mngis1-all-20221009.sql
# Then copy in:
# psql populist-platform-dev < mngis1-all-20221009.sql
