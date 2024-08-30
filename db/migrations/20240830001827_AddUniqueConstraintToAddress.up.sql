-- Add up migration script here
ALTER TABLE address
ADD CONSTRAINT unique_address
UNIQUE (line_1, line_2, city, state, country, postal_code);
