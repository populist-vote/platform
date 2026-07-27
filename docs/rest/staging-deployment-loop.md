# REST staging deployment loop

`scripts/deploy_staging_loop.sh` deploys the current platform and web revisions
to their staging environments and runs live integration checks against both
applications.

The loop targets:

- Heroku app `populist-api-staging`
- `https://api.staging.populist.us`
- Vercel project `populist/web`
- `https://staging.populist.us`

It never targets production. Review both repositories before every run.

## One-time setup

Install and authenticate the deployment CLIs:

```bash
brew install heroku
heroku login
npm install --global vercel
vercel login
```

The default web deployment mode pushes `web/main`, which is the staging trigger
documented by the web repository. The Vercel CLI is needed only for an explicit
direct deployment.

The scripts do not read or write credentials. The Heroku and Vercel CLIs manage
their own authentication. Set `POPULIST_API_KEY` only if the staging gateway
requires bearer authentication; the smoke script sends it as a header and never
prints it.

## Run the complete loop

Start from the platform repository with clean, committed platform and web
working trees:

```bash
./scripts/deploy_staging_loop.sh
```

Pass a number from 1 through 5 to control how many complete smoke-test passes
are attempted after deployment:

```bash
./scripts/deploy_staging_loop.sh 3
```

The loop:

1. Refuses dirty repositories unless `ALLOW_DIRTY=1` is deliberate.
2. Stops when the platform branch contains unapplied migrations relative to
   `origin/main`.
3. Runs `scripts/check_ballot_rest.sh`.
4. Pushes the platform revision to the Heroku staging app.
5. Pushes `web/main` to trigger Vercel staging.
6. Polls for REST and documentation readiness.
7. Tests API discovery, health, states, ballot lookup, address non-disclosure,
   problem responses, client documentation, and the published OpenAPI contract.

Set `SKIP_PREFLIGHT=1` only when the same platform revision has already passed
the deterministic gate.

## Direct Vercel deployment

When no new Git commit exists but staging must be rebuilt, use the authenticated
Vercel CLI. This deploys a preview using the project's staging environment and
then assigns the staging domain:

```bash
WEB_DEPLOY_MODE=vercel ./scripts/deploy_staging_loop.sh
```

Override project settings only when the deployment configuration changes:

```bash
VERCEL_SCOPE=populist \
VERCEL_PROJECT=web \
WEB_STAGING_DOMAIN=staging.populist.us \
WEB_DEPLOY_MODE=vercel \
./scripts/deploy_staging_loop.sh
```

## Test staging without deploying

Run the live verification suite independently:

```bash
./scripts/check_staging_rest.sh
```

The suite uses a public building address—Minneapolis City Hall—rather than a
person's address. It discovers the most recent Minnesota election through
GraphQL. Set a known staging election explicitly when required:

```bash
BALLOT_ELECTION_ID=00000000-0000-0000-0000-000000000000 \
./scripts/check_staging_rest.sh
```

Useful readiness controls:

```bash
WAIT_ATTEMPTS=40 WAIT_SECONDS=15 ./scripts/check_staging_rest.sh
```

On failure, inspect the Heroku release and Vercel deployment logs before
retrying. Do not work around a failing smoke assertion by weakening it without
first reconciling the executable REST contract and client documentation.
