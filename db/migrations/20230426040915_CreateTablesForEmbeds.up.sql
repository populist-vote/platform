-- Add up migration script here
CREATE TABLE IF NOT EXISTS question (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    prompt TEXT NOT NULL,
    response_char_limit INTEGER,
    response_placeholder_text TEXT,
    allow_anonymous_responses BOOLEAN NOT NULL DEFAULT FALSE,
    embed_id UUID REFERENCES embed(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON question
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE IF NOT EXISTS respondent (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON respondent
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE IF NOT EXISTS question_submission (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    question_id UUID NOT NULL REFERENCES question(id) ON DELETE CASCADE,
    respondent_id UUID REFERENCES respondent(id) ON DELETE CASCADE,
    response TEXT NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON question_submission
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

-- Create check to ensure that respondent_id is not null on question_submission if question has allow_anonymous_responses set to false
CREATE OR REPLACE FUNCTION check_question_respondent_id() RETURNS TRIGGER AS $$
    BEGIN
        IF (SELECT allow_anonymous_responses FROM question WHERE id = NEW.question_id) = FALSE THEN
            IF NEW.respondent_id IS NULL THEN
                RAISE EXCEPTION 'respondent_id cannot be null for question with allow_anonymous_responses set to false';
            END IF;
        END IF;
        RETURN NEW;
    END;

$$ LANGUAGE plpgsql;

CREATE TRIGGER check_question_respondent_id_trigger
    BEFORE INSERT OR UPDATE ON question_submission
    FOR EACH ROW EXECUTE PROCEDURE check_question_respondent_id();


CREATE TABLE IF NOT EXISTS poll (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name TEXT,
    prompt TEXT NOT NULL,
    embed_id UUID REFERENCES embed(id) ON DELETE CASCADE,
    allow_anonymous_responses BOOLEAN NOT NULL DEFAULT FALSE,
    allow_write_in_responses BOOLEAN NOT NULL DEFAULT FALSE,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON poll
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE IF NOT EXISTS poll_option (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    poll_id UUID NOT NULL REFERENCES poll(id) ON DELETE CASCADE,
    option_text TEXT NOT NULL,
    is_write_in BOOLEAN NOT NULL DEFAULT FALSE,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON poll_option
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE IF NOT EXISTS poll_submission (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    poll_id UUID NOT NULL REFERENCES poll(id) ON DELETE CASCADE,
    poll_option_id UUID NOT NULL REFERENCES poll_option(id) ON DELETE CASCADE,
    respondent_id UUID REFERENCES respondent(id) ON DELETE CASCADE,
    write_in_response TEXT,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION check_poll_respondent_id() RETURNS TRIGGER AS $$
    BEGIN
        IF (SELECT allow_anonymous_responses FROM poll WHERE id = NEW.poll_id) = FALSE THEN
            IF NEW.respondent_id IS NULL THEN
                RAISE EXCEPTION 'respondent_id cannot be null for poll with allow_anonymous_responses set to false';
            END IF;
        END IF;
        RETURN NEW;
    END;

$$ LANGUAGE plpgsql;

CREATE TRIGGER check_poll_respondent_id_trigger
    BEFORE INSERT OR UPDATE ON poll_submission
    FOR EACH ROW EXECUTE PROCEDURE check_poll_respondent_id();

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON poll_submission
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();
