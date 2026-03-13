#!/usr/bin/env python3
"""
Convert Texas Hart/CIRA Cumulative Election Results PDF to CSV.
Handles proposition, general election, and primary election PDFs.
Outputs in Travis County summary format.

Usage:
    python3 hart_pdf_to_csv.py <input.pdf> [output.csv] [--county NAME]
    python3 hart_pdf_to_csv.py --batch [--county NAME]

Options:
    --county NAME  Specify county name (e.g., --county Matagorda)
    --batch        Process all PDFs in hart_input/ folder, output to hart_output/

Requirements:
    pip3 install pdfplumber pandas
"""

import sys
import re
import pdfplumber
import pandas as pd


# ── Patterns ──────────────────────────────────────────────────────────────────

VOTE_TAIL_8 = r'([\d,]+)\s+([\d.]+%)\s+([\d,]+)\s+([\d.]+%)\s+([\d,]+)\s+([\d.]+%)\s+([\d,]+)\s+([\d.]+%)'

CANDIDATE_WITH_PARTY_RE = re.compile(
    r'^(.+?)\s+(REP|DEM|LIB|GRN|IND|NPA|\(W\))\s+' + VOTE_TAIL_8
)
CANDIDATE_NO_PARTY_RE = re.compile(
    r'^(.+?)\s+' + VOTE_TAIL_8
)
PROP_CHOICE_RE = re.compile(r'^(FOR|AGAINST)\s+' + VOTE_TAIL_8)

SUMMARY_RE = re.compile(
    r'^(Cast Votes:|Undervotes:|Overvotes:|Rejected write-in votes:|Unresolved write-in votes:)\s+'
    r'([\d,]+)\s*([\d.%]*)\s+'
    r'([\d,]+)\s*([\d.%]*)\s+'
    r'([\d,]+)\s*([\d.%]*)\s+'
    r'([\d,]+)\s*([\d.%]*)'
)

HEADER_RE = re.compile(r'^Choice\s+Party\s+(.+)')

# Document-wide precinct line: "X of Y = Z%" following "Precincts Reporting"
PRECINCTS_REPORTING_RE = re.compile(r'^(\d+)\s+of\s+(\d+)\s*=\s*([\d.]+%)')

# Per-race precinct line (Matagorda style): "23 23 100.00% 13,359 22,338 59.80%"
PER_RACE_PRECINCTS_RE = re.compile(r'^(\d+)\s+(\d+)\s+([\d.]+%)\s+([\d,]+)\s+([\d,]+)\s+([\d.]+%)')

SKIP_RE = re.compile(
    r'^(Choice\s|Cumulative Results|Run Time|Run Date|Registered Voters|'
    r'Precincts Reporting|Precincts Counted|Unofficial|Official|Page \d+|\d+ of \d+|'
    r'\d+/\d+/\d+|\*\*\*|PRIMARY ELECTION|GENERAL ELECTION|'
    r'CONSTITUTIONAL AMENDMENT|NOVEMBER|MARCH|JOINT PRIMARY|'
    r'Counted\s+Total\s+Percent|Voters|Ballots\s+Registered)'
)


def parse_column_order(header_rest):
    labels = ['Election Day Voting', 'Absentee Voting', 'Early Voting', 'Total']
    positions = {label: header_rest.find(label) for label in labels}
    ordered = sorted([(pos, label) for label, pos in positions.items() if pos >= 0])
    return [label for _, label in ordered]


def looks_like_race(line):
    if not line or SKIP_RE.match(line):
        return False
    if PROP_CHOICE_RE.match(line):
        return False
    if SUMMARY_RE.match(line):
        return False
    if CANDIDATE_WITH_PARTY_RE.match(line):
        return False
    if PER_RACE_PRECINCTS_RE.match(line):
        return False
    if re.match(r'^\d', line):
        return False
    if re.search(r'\d+\s+[\d.]+%\s*$', line):
        return False
    return True


def clean_name(name):
    return re.sub(r'([A-Za-z]) ([a-z])', r'\1\2', name).strip()


def make_row(race, col_order, choice, party, groups,
             precincts_counted, precincts_total, precincts_pct):
    mapping = {}
    for i, col in enumerate(col_order):
        mapping[col] = (groups[i * 2].replace(',', ''), groups[i * 2 + 1])

    def get(col):
        return mapping.get(col, ('', ''))

    ed_v, ed_p = get('Election Day Voting')
    ab_v, ab_p = get('Absentee Voting')
    ev_v, ev_p = get('Early Voting')
    tot_v, tot_p = get('Total')

    return {
        'Race':               race,
        'Choice':             choice,
        'Party':              party,
        'ElectionDay_Votes':  ed_v,
        'ElectionDay_Pct':    ed_p,
        'Absentee_Votes':     ab_v,
        'Absentee_Pct':       ab_p,
        'Early_Votes':        ev_v,
        'Early_Pct':          ev_p,
        'Total_Votes':        tot_v,
        'Total_Pct':          tot_p,
        'Precincts_Counted':  precincts_counted,
        'Precincts_Total':    precincts_total,
        'Precincts_Pct':      precincts_pct,
    }


def transform_to_travis_format(rows, county_name=""):
    """
    Transform CIRA format rows to Travis County summary format.

    Travis format columns:
    - line number
    - contest name
    - choice name
    - party name
    - total votes
    - percent of votes
    - registered voters
    - ballots cast
    - num Precinct total
    - num Precinct rptg
    - over votes
    - under votes
    - county
    """
    transformed = []

    for idx, row in enumerate(rows, start=1):
        # Determine if this is an overvotes or undervotes row
        choice_lower = row.get('Choice', '').lower()
        is_overvote = 'overvote' in choice_lower
        is_undervote = 'undervote' in choice_lower

        transformed_row = {
            'line number': idx,
            'contest name': row.get('Race', ''),
            'choice name': row.get('Choice', ''),
            'party name': row.get('Party', ''),
            'total votes': row.get('Total_Votes', ''),
            'percent of votes': row.get('Total_Pct', '').replace('%', ''),
            'registered voters': '',  # Not available in CIRA format
            'ballots cast': '',  # Not available in CIRA format
            'num Precinct total': row.get('Precincts_Total', ''),
            'num Precinct rptg': row.get('Precincts_Counted', ''),
            'over votes': row.get('Total_Votes', '') if is_overvote else '',
            'under votes': row.get('Total_Votes', '') if is_undervote else '',
            'county': county_name
        }

        transformed.append(transformed_row)

    return transformed


def parse_results(pdf_path, csv_path=None, county_name=""):
    if csv_path is None:
        csv_path = pdf_path.rsplit('.', 1)[0] + '.csv'

    rows = []
    current_race = None
    col_order = ['Absentee Voting', 'Early Voting', 'Election Day Voting', 'Total']

    # Document-wide precinct info (from page headers)
    doc_precincts_counted = ''
    doc_precincts_total = ''
    doc_precincts_pct = ''

    # Per-race precinct info (Matagorda style)
    race_precincts_counted = ''
    race_precincts_total = ''
    race_precincts_pct = ''

    # Track whether we just saw the "Precincts Reporting" label
    # so we know the next "X of Y = Z%" line is doc-wide
    saw_precincts_reporting_label = False
    # Track whether we just saw the per-race precincts header
    saw_per_race_header = False

    with pdfplumber.open(pdf_path) as pdf:
        # Try to extract county name from first page header if not provided
        if not county_name and len(pdf.pages) > 0:
            first_page_text = pdf.pages[0].extract_text()
            if first_page_text:
                # Check first few lines for county name
                for line in first_page_text.splitlines()[:10]:
                    # Look for "COUNTY, TEXAS" pattern
                    match = re.search(r'COUNTY,\s+TEXAS', line, re.IGNORECASE)
                    if match:
                        # Get text before "COUNTY, TEXAS"
                        before_county = line[:match.start()].strip()
                        words = before_county.split()

                        if words:
                            # Take last word, or last 2 words if the second-to-last is
                            # a common multi-word county prefix (San, El, De, etc.)
                            if len(words) >= 2 and words[-2].upper() in ['SAN', 'EL', 'DE', 'LA', 'VAN']:
                                county_name = ' '.join(words[-2:]).title()
                            else:
                                county_name = words[-1].title()

                            print(f'Detected county: {county_name}')
                            break
        for page in pdf.pages:
            text = page.extract_text()
            if not text:
                continue

            for raw_line in text.splitlines():
                line = raw_line.strip()
                if not line:
                    continue

                # ── Detect "Precincts Reporting" label (doc-wide) ──
                if re.match(r'^Precincts Reporting\s*$', line):
                    saw_precincts_reporting_label = True
                    continue

                # ── Capture doc-wide "X of Y = Z%" ──
                # Stay in saw_precincts_reporting_label state through short non-data lines
                # (e.g. "Election" injected by pdfplumber from multi-line header)
                if saw_precincts_reporting_label:
                    m = PRECINCTS_REPORTING_RE.match(line)
                    if not m:
                        # Skip short noise lines and keep waiting
                        if len(line.split()) <= 3 and not re.match(r'^\d', line):
                            continue
                        # Something else — give up
                        saw_precincts_reporting_label = False
                        # Fall through to normal processing
                    if m:
                        saw_precincts_reporting_label = False
                        doc_precincts_counted = m.group(1)
                        doc_precincts_total = m.group(2)
                        doc_precincts_pct = m.group(3)
                        # Reset per-race since this is a new doc-wide value
                        race_precincts_counted = ''
                        race_precincts_total = ''
                        race_precincts_pct = ''
                        continue

                # ── Detect per-race precincts header (Matagorda style) ──
                if re.match(r'^Precincts\s+Voters\s*$', line):
                    saw_per_race_header = True
                    continue
                if saw_per_race_header and re.match(r'^Counted\s+Total\s+Percent', line):
                    continue  # skip the column label row
                if saw_per_race_header:
                    saw_per_race_header = False
                    m = PER_RACE_PRECINCTS_RE.match(line)
                    if m:
                        race_precincts_counted = m.group(1)
                        race_precincts_total = m.group(2)
                        race_precincts_pct = m.group(3)
                        continue

                # ── Column header ──
                m = HEADER_RE.match(line)
                if m:
                    col_order = parse_column_order(m.group(1))
                    continue

                # ── Race title ──
                if looks_like_race(line):
                    current_race = line
                    # Reset per-race precincts for new race
                    race_precincts_counted = ''
                    race_precincts_total = ''
                    race_precincts_pct = ''
                    continue

                if not current_race:
                    continue

                # Resolve which precinct info to use: per-race takes priority
                p_counted = race_precincts_counted or doc_precincts_counted
                p_total   = race_precincts_total   or doc_precincts_total
                p_pct     = race_precincts_pct     or doc_precincts_pct

                # ── FOR / AGAINST ──
                m = PROP_CHOICE_RE.match(line)
                if m:
                    rows.append(make_row(current_race, col_order, m.group(1), '',
                                         m.groups()[1:], p_counted, p_total, p_pct))
                    continue

                # ── Summary rows ──
                m = SUMMARY_RE.match(line)
                if m:
                    vals = [m.group(2), m.group(3), m.group(4), m.group(5),
                            m.group(6), m.group(7), m.group(8), m.group(9)]
                    rows.append(make_row(current_race, col_order,
                                         m.group(1).rstrip(':'), '', vals,
                                         p_counted, p_total, p_pct))
                    continue

                # ── Candidate with party ──
                m = CANDIDATE_WITH_PARTY_RE.match(line)
                if m:
                    name = clean_name(m.group(1))
                    rows.append(make_row(current_race, col_order, name, m.group(2),
                                         m.groups()[2:], p_counted, p_total, p_pct))
                    continue

                # ── Candidate without party ──
                m = CANDIDATE_NO_PARTY_RE.match(line)
                if m:
                    name = clean_name(m.group(1))
                    rows.append(make_row(current_race, col_order, name, '',
                                         m.groups()[1:], p_counted, p_total, p_pct))
                    continue

    print(f'Found {len(rows)} rows')

    # Transform to Travis County format
    transformed_rows = transform_to_travis_format(rows, county_name)

    travis_cols = [
        'line number', 'contest name', 'choice name', 'party name',
        'total votes', 'percent of votes', 'registered voters', 'ballots cast',
        'num Precinct total', 'num Precinct rptg', 'over votes', 'under votes', 'county'
    ]

    df = pd.DataFrame(transformed_rows, columns=travis_cols)
    df.to_csv(csv_path, index=False)
    print(f'Saved to: {csv_path}')
    return df


if __name__ == '__main__':
    # Parse command line arguments
    args = sys.argv[1:]
    pdf_path = None
    csv_path = None
    county_name = ""
    batch_mode = False

    i = 0
    while i < len(args):
        arg = args[i]
        if arg == '--batch':
            batch_mode = True
        elif arg == '--county':
            if i + 1 < len(args):
                county_name = args[i + 1]
                i += 1  # Skip next arg
            else:
                print('Error: --county requires a value')
                sys.exit(1)
        elif arg.startswith('--'):
            print(f'Unknown option: {arg}')
            print(__doc__)
            sys.exit(1)
        elif pdf_path is None:
            pdf_path = arg
        elif csv_path is None:
            csv_path = arg
        else:
            print('Too many arguments')
            print(__doc__)
            sys.exit(1)
        i += 1

    # Handle batch mode
    if batch_mode:
        import glob
        import os

        # Get script directory
        script_dir = os.path.dirname(os.path.abspath(__file__))
        input_dir = os.path.join(script_dir, 'hart_input')
        output_dir = os.path.join(script_dir, 'hart_output')

        # Create directories if they don't exist
        if not os.path.exists(input_dir):
            os.makedirs(input_dir)
            print(f'Created directory: {input_dir}')
            print('Please place PDF files in this directory and run again.')
            sys.exit(0)

        if not os.path.exists(output_dir):
            os.makedirs(output_dir)
            print(f'Created directory: {output_dir}')

        # Find all PDF files in input directory
        pdf_files = glob.glob(os.path.join(input_dir, '*.pdf'))

        if not pdf_files:
            print(f'No PDF files found in {input_dir}')
            print('Please place PDF files in this directory and run again.')
            sys.exit(0)

        print(f'\nFound {len(pdf_files)} PDF file(s) to process')
        print('=' * 60)

        # Process each PDF
        success_count = 0
        fail_count = 0

        for pdf_file in pdf_files:
            basename = os.path.basename(pdf_file)
            csv_filename = basename.rsplit('.', 1)[0] + '.csv'
            output_csv = os.path.join(output_dir, csv_filename)

            print(f'\nProcessing: {basename}')
            print('-' * 60)

            try:
                result = parse_results(pdf_file, output_csv, county_name)
                if result is not None:
                    success_count += 1
                    print(f'✓ Success: {csv_filename}')
                else:
                    fail_count += 1
                    print(f'✗ Failed: {basename}')
            except Exception as e:
                fail_count += 1
                print(f'✗ Error processing {basename}: {e}')

        print('\n' + '=' * 60)
        print(f'Batch processing complete!')
        print(f'  Successful: {success_count}')
        print(f'  Failed: {fail_count}')
        print(f'  Output directory: {output_dir}')

        sys.exit(0 if fail_count == 0 else 1)

    # Single file mode
    if len(args) < 1 or pdf_path is None:
        print(__doc__)
        sys.exit(1)

    parse_results(pdf_path, csv_path, county_name)
