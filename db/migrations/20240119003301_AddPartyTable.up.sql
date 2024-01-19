CREATE TABLE party (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    fec_code text,
    name text NOT NULL,
    description text,
    notes text
);

INSERT INTO party (fec_code, name, notes)
VALUES
(NULL, 'Approval Voting Party', NULL),
(NULL, 'Grassroots-Legalize Cannabis', NULL),
(NULL, 'Legal Marijuana Now', NULL),
(NULL, 'Colorado Center', NULL),
(NULL, 'Unity', NULL),

('ACE', 'Ace Party', NULL),
('AKI', 'Alaskan Independence Party', NULL),
('AIC', 'American Independent Conservative', NULL),
('AIP', 'American Independent Party', NULL),
('AMP', 'American Party', NULL),
('APF', 'American Peoples Freedom Party', NULL),
('AE', 'Americans Elect', NULL),
('CIT', 'Citizens Party', NULL),
('CMD', 'Commandments Party', NULL),
('CMP', 'Commonwealth Party of the U.S.', NULL),
('COM', 'Communist Party', NULL),
('CNC', 'Concerned Citizens Party Of Connecticut', NULL),
('CRV', 'Conservative Party', NULL),
('CON', 'Constitution Party', NULL),
('CST', 'Constitutional', NULL),
('COU', 'Country', NULL),
('DCG', 'D.C. Statehood Green Party', NULL),
('DNL', 'Democratic -Nonpartisan League', NULL),
('DEM', 'Democratic Party', NULL),
('D/C', 'Democratic/Conservative', NULL),
('DFL', 'Democratic-Farmer-Labor', NULL),
('DGR', 'Desert Green Party', NULL),
('FED', 'Federalist', NULL),
('FLP', 'Freedom Labor Party', NULL),
('FRE', 'Freedom Party', NULL),
('GWP', 'George Wallace Party', NULL),
('GRT', 'Grassroots', NULL),
('GRE', 'Green Party', NULL),
('GR', 'Green-Rainbow', NULL),
('HRP', 'Human Rights Party', NULL),
('IDP', 'Independence Party', NULL),
('IND', 'Independent', NULL),
('IAP', 'Independent American Party', NULL),
('ICD', 'Independent Conservative Democratic', NULL),
('IGR', 'Independent Green', NULL),
('IP', 'Independent Party', NULL),
('IDE', 'Independent Party of Delaware', NULL),
('IGD', 'Industrial Government Party', NULL),
('JCN', 'Jewish/Christian National', NULL),
('JUS', 'Justice Party', NULL),
('LRU', 'La Raza Unida', 'Also see RUP'),
('LBR', 'Labor Party', 'Also see LAB'),
('LFT', 'Less Federal Taxes', NULL),
('LBL', 'Liberal Party', NULL),
('LIB', 'Libertarian Party', NULL),
('LBU', 'Liberty Union Party', NULL),
('MTP', 'Mountain Party', NULL),
('NDP', 'National Democratic Party', NULL),
('NLP', 'Natural Law Party', NULL),
('NA', 'New Alliance', NULL),
('NJC', 'New Jersey Conservative Party', NULL),
('NPP', 'New Progressive Party', NULL),
('NPA', 'No Party Affiliation', NULL),
('NOP', 'No Party Preference', 'Commonly used in CA & WA'),
('NNE', 'None', NULL),
('N', 'Nonpartisan', NULL),
('NON', 'Non-Party', NULL),
('OE', 'One Earth Party', NULL),
('OTH', 'Other', NULL),
('PG', 'Pacific Green', NULL),
('PSL', 'Party for Socialism and Liberation', NULL),
('PAF', 'Peace And Freedom', 'Also see PFP'),
('PFP', 'Peace And Freedom Party', 'Also see PAF'),
('PFD', 'Peace Freedom Party', NULL),
('POP', 'People Over Politics', NULL),
('PPY', 'People''s Party', NULL),
('PCH', 'Personal Choice Party', NULL),
('PPD', 'Popular Democratic Party', NULL),
('PRO', 'Progressive Party', NULL),
('NAP', 'Prohibition Party', NULL),
('PRI', 'Puerto Rican Independence Party', NULL),
('RUP', 'Raza Unida Party', 'Also see LRU'),
('REF', 'Reform Party', NULL),
('REP', 'Republican Party', NULL),
('RES', 'Resource Party', NULL),
('RTL', 'Right To Life', NULL),
('SEP', 'Socialist Equality Party', NULL),
('SLP', 'Socialist Labor Party', NULL),
('SUS', 'Socialist Party', NULL),
('SOC', 'Socialist Party U.S.A.', NULL),
('SWP', 'Socialist Workers Party', NULL),
('TX', 'Taxpayers', NULL),
('TWR', 'Taxpayers Without Representation', NULL),
('TEA', 'Tea Party', NULL),
('THD', 'Theo-Democratic', NULL),
('LAB', 'U.S. Labor Party', 'Also see LBR'),
('USP', 'U.S. People''s Party', NULL),
('UST', 'U.S. Taxpayers Party', NULL),
('UN', 'Unaffiliated', NULL),
('UC', 'United Citizen', NULL),
('UNI', 'United Party', NULL),
('UNK', 'Unknown', NULL),
('VET', 'Veterans Party', NULL),
('WTP', 'We the People', NULL),
('W', 'Write-In', NULL);

ALTER TABLE politician ADD COLUMN party_id uuid REFERENCES party (id);
ALTER TABLE race ADD COLUMN party_id uuid REFERENCES party (id);
ALTER TABLE user_profile ADD COLUMN party_id uuid REFERENCES party (id);

UPDATE politician
SET
    party_id = (CASE
        WHEN
            party = 'democratic'
            THEN (SELECT id FROM party WHERE fec_code = 'DEM')
        WHEN
            party = 'republican'
            THEN (SELECT id FROM party WHERE fec_code = 'REP')
        WHEN
            party = 'independent'
            THEN (SELECT id FROM party WHERE fec_code = 'IND')
        WHEN party = 'green' THEN (SELECT id FROM party WHERE fec_code = 'GRE')
        WHEN
            party = 'libertarian'
            THEN (SELECT id FROM party WHERE fec_code = 'LIB')
        WHEN
            party = 'american_constitution'
            THEN (SELECT id FROM party WHERE fec_code = 'CON')
        WHEN
            party = 'freedom'
            THEN (SELECT id FROM party WHERE fec_code = 'FRE')
        WHEN party = 'unity' THEN (SELECT id FROM party WHERE name = 'Unity')
        WHEN
            party = 'approval_voting'
            THEN (SELECT id FROM party WHERE name = 'Approval Voting Party')
        WHEN
            party = 'democratic_farmer_labor'
            THEN (SELECT id FROM party WHERE fec_code = 'DFL')
        WHEN
            party = 'grassroots_legalize_cannabis'
            THEN
                (
                    SELECT id
                    FROM party
                    WHERE name = 'Grassroots-Legalize Cannabis'
                )
        WHEN
            party = 'legal_marijuana_now'
            THEN (SELECT id FROM party WHERE name = 'Legal Marijuana Now')
        WHEN
            party = 'socialist_workers'
            THEN (SELECT id FROM party WHERE fec_code = 'SWP')
        WHEN
            party = 'colorado_center'
            THEN (SELECT id FROM party WHERE name = 'Colorado Center')
        WHEN
            party = 'unknown'
            THEN (SELECT id FROM party WHERE fec_code = 'UNK')
        WHEN
            party = 'unaffiliated'
            THEN (SELECT id FROM party WHERE fec_code = 'UN')
        ELSE (SELECT id FROM party WHERE fec_code = 'OTH')
    END);

UPDATE race
SET
    party_id = (CASE
        WHEN
            party = 'democratic'
            THEN (SELECT id FROM party WHERE fec_code = 'DEM')
        WHEN
            party = 'republican'
            THEN (SELECT id FROM party WHERE fec_code = 'REP')
        WHEN
            party = 'independent'
            THEN (SELECT id FROM party WHERE fec_code = 'IND')
        WHEN party = 'green' THEN (SELECT id FROM party WHERE fec_code = 'GRE')
        WHEN
            party = 'libertarian'
            THEN (SELECT id FROM party WHERE fec_code = 'LIB')
        WHEN
            party = 'american_constitution'
            THEN (SELECT id FROM party WHERE fec_code = 'CON')
        WHEN
            party = 'freedom'
            THEN (SELECT id FROM party WHERE fec_code = 'FRE')
        WHEN party = 'unity' THEN (SELECT id FROM party WHERE name = 'Unity')
        WHEN
            party = 'approval_voting'
            THEN (SELECT id FROM party WHERE name = 'Approval Voting Party')
        WHEN
            party = 'democratic_farmer_labor'
            THEN (SELECT id FROM party WHERE fec_code = 'DFL')
        WHEN
            party = 'grassroots_legalize_cannabis'
            THEN
                (
                    SELECT id
                    FROM party
                    WHERE name = 'Grassroots-Legalize Cannabis'
                )
        WHEN
            party = 'legal_marijuana_now'
            THEN (SELECT id FROM party WHERE name = 'Legal Marijuana Now')
        WHEN
            party = 'socialist_workers'
            THEN (SELECT id FROM party WHERE fec_code = 'SWP')
        WHEN
            party = 'colorado_center'
            THEN (SELECT id FROM party WHERE name = 'Colorado Center')
        WHEN
            party = 'unknown'
            THEN (SELECT id FROM party WHERE fec_code = 'UNK')
        WHEN
            party = 'unaffiliated'
            THEN (SELECT id FROM party WHERE fec_code = 'UN')
        ELSE (SELECT id FROM party WHERE fec_code = 'OTH')
    END);

UPDATE user_profile
SET
    party_id = (CASE
        WHEN
            party = 'democratic'
            THEN (SELECT id FROM party WHERE fec_code = 'DEM')
        WHEN
            party = 'republican'
            THEN (SELECT id FROM party WHERE fec_code = 'REP')
        WHEN
            party = 'independent'
            THEN (SELECT id FROM party WHERE fec_code = 'IND')
        WHEN party = 'green' THEN (SELECT id FROM party WHERE fec_code = 'GRE')
        WHEN
            party = 'libertarian'
            THEN (SELECT id FROM party WHERE fec_code = 'LIB')
        WHEN
            party = 'american_constitution'
            THEN (SELECT id FROM party WHERE fec_code = 'CON')
        WHEN
            party = 'freedom'
            THEN (SELECT id FROM party WHERE fec_code = 'FRE')
        WHEN party = 'unity' THEN (SELECT id FROM party WHERE name = 'Unity')
        WHEN
            party = 'approval_voting'
            THEN (SELECT id FROM party WHERE name = 'Approval Voting Party')
        WHEN
            party = 'democratic_farmer_labor'
            THEN (SELECT id FROM party WHERE fec_code = 'DFL')
        WHEN
            party = 'grassroots_legalize_cannabis'
            THEN
                (
                    SELECT id
                    FROM party
                    WHERE name = 'Grassroots-Legalize Cannabis'
                )
        WHEN
            party = 'legal_marijuana_now'
            THEN (SELECT id FROM party WHERE name = 'Legal Marijuana Now')
        WHEN
            party = 'socialist_workers'
            THEN (SELECT id FROM party WHERE fec_code = 'SWP')
        WHEN
            party = 'colorado_center'
            THEN (SELECT id FROM party WHERE name = 'Colorado Center')
        WHEN
            party = 'unknown'
            THEN (SELECT id FROM party WHERE fec_code = 'UNK')
        WHEN
            party = 'unaffiliated'
            THEN (SELECT id FROM party WHERE fec_code = 'UN')
        ELSE (SELECT id FROM party WHERE fec_code = 'OTH')
    END);
