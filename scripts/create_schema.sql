-- SCHEMA: p6t_state_mn

-- DROP SCHEMA p6t_state_mn ;

CREATE SCHEMA p6t_state_mn
    AUTHORIZATION postgres;

COMMENT ON SCHEMA p6t_state_mn
    IS 'MN state data and GIS shapefiles';

GRANT ALL ON SCHEMA p6t_state_mn TO PUBLIC;

GRANT ALL ON SCHEMA p6t_state_mn TO postgres;
