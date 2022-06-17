ALTER TABLE user_profile
DROP CONSTRAINT fk_user,
DROP CONSTRAINT fk_address,
ADD CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES populist_user(id) ON DELETE CASCADE,
ADD CONSTRAINT fk_address FOREIGN KEY(address_id) REFERENCES address(id) ON DELETE CASCADE;

ALTER TABLE voting_guide
DROP CONSTRAINT fk_user,
ADD CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES populist_user(id) ON DELETE CASCADE;

ALTER TABLE voting_guide_candidates
DROP CONSTRAINT fk_voting_guide,
ADD CONSTRAINT fk_voting_guide FOREIGN KEY(voting_guide_id) REFERENCES voting_guide(id) ON DELETE CASCADE;