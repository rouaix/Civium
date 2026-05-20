-- Migration 015 : network_id dans web_notifications

ALTER TABLE web_notifications
    ADD COLUMN network_id VARCHAR(64) NOT NULL DEFAULT 'civium-principal-000000000000000000000000000000000',
    ADD INDEX idx_web_notif_network (network_id)
