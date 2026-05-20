-- Migration 009 : network_id dans web_messages

ALTER TABLE web_messages
    ADD COLUMN network_id VARCHAR(64) NOT NULL DEFAULT 'civium-principal-000000000000000000000000000000000',
    ADD INDEX idx_web_messages_network (network_id)
