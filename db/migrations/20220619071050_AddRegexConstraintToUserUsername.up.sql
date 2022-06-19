-- Add up migration script here
ALTER TABLE populist_user ADD CONSTRAINT check_valid_chars CHECK ( username ~ '^(?=.{3,20}$)(?![_.])(?!.*[_.]{2})[a-zA-Z0-9._]+(?<![_.])$' );