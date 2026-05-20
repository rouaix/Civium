-- Migration 014 : network_id dans web_activity

ALTER TABLE web_activity
    ADD COLUMN network_id VARCHAR(64) NOT NULL DEFAULT 'civium-principal-000000000000000000000000000000000',
    ADD INDEX idx_web_activity_network (network_id)
