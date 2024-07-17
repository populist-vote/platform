-- Add down migration script here
CREATE TYPE user_role AS ENUM ('superuser', 'staff', 'premium', 'basic');

DROP INDEX organization_users_organization_idx;
DROP TABLE public.organization_users;
DROP TYPE organization_role_type;

ALTER TABLE public.populist_user
ADD COLUMN organization_id uuid
REFERENCES public.organization
ON DELETE CASCADE ON UPDATE CASCADE,
ADD COLUMN role user_role NOT NULL DEFAULT 'basic',
DROP COLUMN system_role;

DROP TYPE system_role_type;
