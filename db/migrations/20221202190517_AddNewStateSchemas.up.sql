-- Add up migration script here


-- Colorado
CREATE SCHEMA p6t_state_co;

COMMENT ON SCHEMA p6t_state_co
    IS 'CO state data and GIS shapefiles';

GRANT ALL ON SCHEMA p6t_state_co TO PUBLIC;

-- California
CREATE SCHEMA p6t_state_ca;

COMMENT ON SCHEMA p6t_state_ca
    IS 'CA state data and GIS shapefiles';

GRANT ALL ON SCHEMA p6t_state_ca TO PUBLIC;

-- Oregon
CREATE SCHEMA p6t_state_or;

COMMENT ON SCHEMA p6t_state_or
    IS 'OR state data and GIS shapefiles';

GRANT ALL ON SCHEMA p6t_state_or TO PUBLIC;

