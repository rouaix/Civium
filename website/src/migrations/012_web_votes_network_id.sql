-- Migration 012 : network_id dans web_votes

ALTER TABLE web_votes
    ADD COLUMN network_id VARCHAR(64) NOT NULL DEFAULT 'civium-principal-000000000000000000000000000000000',
    ADD INDEX idx_web_votes_network (network_id)
