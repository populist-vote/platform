-- Add up migration script here
CREATE UNIQUE INDEX idx_bill_id_legiscan_bill_id ON bill (id, legiscan_bill_id);
