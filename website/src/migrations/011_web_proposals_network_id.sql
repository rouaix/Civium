-- Migration 011 : network_id dans web_proposals

ALTER TABLE web_proposals
    ADD COLUMN network_id VARCHAR(64) NOT NULL DEFAULT 'civium-principal-000000000000000000000000000000000',
    ADD INDEX idx_web_proposals_network (network_id)
