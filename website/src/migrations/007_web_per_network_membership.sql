-- Migration 007 : network_id dans web_members

ALTER TABLE web_members
    ADD COLUMN network_id VARCHAR(64) NOT NULL DEFAULT 'civium-principal-000000000000000000000000000000000',
    ADD INDEX idx_web_members_network (network_id)
