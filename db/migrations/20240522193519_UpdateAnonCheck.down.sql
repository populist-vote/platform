-- Add down migration script here
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
