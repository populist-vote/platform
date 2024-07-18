-- Add up migration script here

CREATE TYPE system_role_type AS ENUM (
    'user', 'staff', 'superuser'
);


CREATE TYPE organization_role_type AS ENUM (
    'read-only', 'member', 'admin', 'owner'
);

CREATE TABLE public.organization_users
(
    user_id uuid NOT NULL
    REFERENCES public.populist_user (id)
    ON DELETE CASCADE
    ON UPDATE CASCADE,

    organization_id uuid NOT NULL
    REFERENCES public.organization
    ON DELETE CASCADE
    ON UPDATE CASCADE,

    role organization_role_type NOT NULL,

    PRIMARY KEY (user_id, organization_id)
);

CREATE INDEX organization_users_organization_idx ON public.organization_users (
    organization_id
);


INSERT INTO public.organization_users (user_id, organization_id, role)
SELECT
    id,
    organization_id,
    'member'::organization_role_type
FROM public.populist_user
WHERE organization_id IS NOT NULL;

ALTER TABLE public.populist_user
ADD COLUMN system_role system_role_type NOT NULL DEFAULT 'user',
DROP COLUMN organization_id,
DROP COLUMN role;

DROP TYPE user_role;
