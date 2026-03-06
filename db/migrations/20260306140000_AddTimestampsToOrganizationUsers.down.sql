DROP TRIGGER set_updated_at ON public.organization_users;

ALTER TABLE public.organization_users
DROP COLUMN updated_at,
DROP COLUMN created_at;
