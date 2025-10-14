# Cleanup Stale Embed Origins Script

## Overview

This script validates and cleans up stale records in the `embed_origin` table by checking if each embed is still present on its associated URL.

## Problem It Solves

When embeds are removed from websites, the `embed_origin` table retains the old records because the `ping_embed_origin` mutation only inserts/updates records, never deletes them. This leads to:

- "More Info" links pointing to pages without embeds
- Stale data accumulating in the database
- Misleading analytics about embed usage

## How It Works

The script:

1. Fetches all records from the `embed_origin` table
2. For each record, makes an HTTP request to the URL
3. Extracts the page title from the `<head>` section only, using multiple strategies (in order of priority):
   - HTML `<title>` tag within `<head>`
   - Meta tag with exact `name="title"`
   - Other meta tags with "title" in the name/property (e.g., `og:title`, `twitter:title`)
4. Scans the HTML content for the embed ID using multiple patterns:
   - Direct embed ID presence in the HTML
   - Populist embed script tags with the ID
   - iframes containing the embed ID
   - Data attributes with the embed ID
   - Div elements with populist classes/IDs containing the embed ID
5. **Updates page titles** if they've changed (in both dry-run and production mode)
6. If the embed is NOT found, marks the record for deletion
7. If the page returns 404, marks the record for deletion
8. Deletes invalid records (unless running in dry-run mode)
9. Provides a detailed summary of results

## Usage

### Dry Run (Recommended First)

Test the script without making any changes:

```bash
cd platform/scripts
cargo run --bin cleanup_stale_embed_origins -- --dry-run
```

This will:

- ✅ Check all URLs
- ✅ Show which records would be deleted
- ✅ Show which page titles would be updated
- ❌ NOT update page titles
- ❌ NOT delete anything from the database

### Production Run

Actually delete stale records and update page titles:

```bash
cd platform/scripts
cargo run --bin cleanup_stale_embed_origins
```

This will:

- ✅ Check all URLs
- ✅ Delete stale records
- ✅ Delete 404 pages
- ✅ Update page titles that have changed

### Verbose Mode

See detailed information about title updates:

```bash
cd platform/scripts
cargo run --bin cleanup_stale_embed_origins -- --dry-run --verbose
# or
cargo run --bin cleanup_stale_embed_origins -- --dry-run -v
```

This will show:

- Each URL where the title is being updated
- Old title vs new title
- Number of rows affected by each update

## Output

The script provides:

- **Progress bar** showing real-time progress
- **Summary statistics**:
  - Valid embeds (still present on pages)
  - Stale embeds (no longer present)
  - Errors (failed to check)
- **List of URLs** that will be/were deleted
- **Total execution time**

### Example Output

```
🔍 Scanning and cleaning up stale embed origins

📊 Found 150 embed origin records to check

[████████████████████████████████████████] 150/150 (00:02:30)

════════════════════════════════════════════════════════════
Summary
════════════════════════════════════════════════════════════
✅ Valid embeds:        142 (94.7%)
❌ Stale embeds:        5 (3.3%)
🚫 Pages not found:     1 (0.7%)
⚠️  Other errors:       2

📊 Total to delete:     6 (4.0%)
📝 Titles updated:      23
════════════════════════════════════════════════════════════

🗑️ 6 records deleted

🕑 Completed in 2m 35s
```

## Error Handling

The script handles various error scenarios:

- **HTTP errors** (404, 500, etc.): Logged but don't stop execution
- **Timeout**: 10-second timeout per URL to prevent hanging
- **Network errors**: Logged and counted in error statistics
- **Database errors**: Logged with details

## Scheduling

### Recommended Schedule

Run this script:

- **Weekly** for active sites with frequent embed changes
- **Monthly** for stable sites
- **After major site updates** where embeds might be removed

### Cron Example

```bash
# Run every Sunday at 2 AM
0 2 * * 0 cd /path/to/platform/scripts && cargo run --bin cleanup_stale_embed_origins >> /var/log/embed_cleanup.log 2>&1
```

## Performance Considerations

- **Rate limiting**: The script checks URLs sequentially to avoid overwhelming servers
- **Timeout**: 10-second timeout per URL prevents hanging on slow sites
- **User agent**: Identifies as "PopulistBot" for transparency
- **Memory**: Minimal memory usage, processes records one at a time

## Safety Features

1. **Dry-run mode**: Test before making changes
2. **Detailed logging**: See exactly what will be deleted
3. **Error isolation**: One failed URL doesn't stop the entire process
4. **Progress tracking**: Monitor progress in real-time

## Complementary Solutions

This script works well with other approaches:

1. **Backend filter** (immediate fix):

   ```rust
   // In platform/graphql/src/types/embed.rs
   AND last_ping_at > NOW() - INTERVAL '7 days'
   ```

2. **Frontend filter** (additional safety):

   ```typescript
   // Filter out origins not pinged recently
   const activeOrigins = embed.origins?.filter((origin) =>
     isRecent(origin.last_ping_at)
   );
   ```

3. **This script** (periodic cleanup):
   - Validates actual embed presence
   - Removes confirmed stale records
   - Keeps database clean

## Troubleshooting

### Script hangs on certain URLs

- Some URLs may be slow to respond
- The 10-second timeout should prevent hanging
- Check the console for the last URL being processed

### High error rate

- Check network connectivity
- Verify URLs are publicly accessible
- Some sites may block automated requests

### False positives (valid embeds marked as stale)

- The embed ID might be loaded dynamically via JavaScript
- Consider adding the URL to a whitelist
- Manually verify the embed is actually present

## Future Enhancements

Possible improvements:

- [ ] JavaScript rendering for SPAs (using headless browser)
- [ ] Parallel processing with rate limiting
- [ ] Whitelist for known false positives
- [ ] Slack/email notifications for summary
- [ ] Export deleted records to CSV for review
- [ ] Automatic retry for network errors

## Related Files

- `/platform/graphql/src/mutation/embed.rs` - `ping_embed_origin` mutation
- `/platform/graphql/src/types/embed.rs` - `origins` resolver
- `/web/components/MyBallotEmbed/MyBallotEmbed.tsx` - `RelatedEmbedLinks` component
