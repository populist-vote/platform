ALTER TABLE public.organization_users
ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc');

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON public.organization_users
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();
