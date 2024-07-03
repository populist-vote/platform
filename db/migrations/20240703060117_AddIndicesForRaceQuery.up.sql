-- Add up migration script here
CREATE INDEX idx_race_office_id ON race (office_id);
CREATE INDEX idx_race_election_id ON race (election_id);
CREATE INDEX idx_race_state ON race (state);
CREATE INDEX idx_office_id ON office (id);
CREATE INDEX idx_office_title_political_scope_election_scope ON office (
    title, political_scope, election_scope
);
CREATE INDEX idx_election_id ON election (id);
CREATE INDEX idx_election_date ON election (election_date);
CREATE INDEX idx_us_states_code ON us_states (code);
