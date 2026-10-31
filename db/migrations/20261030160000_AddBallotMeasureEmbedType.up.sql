ALTER TYPE embed_type ADD VALUE 'ballot_measure';

CREATE OR REPLACE FUNCTION delete_embeds_on_ballot_measure_delete()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM embed
    WHERE attributes->>'ballotMeasureId' = OLD.id::text;

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_delete_embeds_on_ballot_measure_delete
AFTER DELETE ON ballot_measure
FOR EACH ROW
EXECUTE FUNCTION delete_embeds_on_ballot_measure_delete();
