-- Migration 010 : network_id dans web_direct_messages

ALTER TABLE web_direct_messages
    ADD COLUMN network_id VARCHAR(64) NOT NULL DEFAULT 'civium-principal-000000000000000000000000000000000',
    ADD INDEX idx_web_dm_network (network_id)
