-- Add up migration script here
ALTER TABLE politician
ADD COLUMN legiscan_people_id INT;
