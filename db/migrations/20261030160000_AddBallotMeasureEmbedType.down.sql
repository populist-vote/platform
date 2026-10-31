DROP TRIGGER IF EXISTS trg_delete_embeds_on_ballot_measure_delete ON ballot_measure;
DROP FUNCTION IF EXISTS delete_embeds_on_ballot_measure_delete();

CREATE TYPE embed_type_new AS ENUM (
    'legislation',
    'legislation_tracker',
    'politician',
    'question',
    'poll',
    'race',
    'candidate_guide',
    'my_ballot',
    'conversation'
);

ALTER TABLE embed ALTER COLUMN embed_type
TYPE embed_type_new USING embed_type::text::embed_type_new;

DROP TYPE embed_type CASCADE;
ALTER TYPE embed_type_new RENAME TO embed_type;
