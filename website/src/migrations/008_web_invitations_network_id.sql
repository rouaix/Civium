-- Migration 008 : network_id dans web_invitations

ALTER TABLE web_invitations
    ADD COLUMN network_id VARCHAR(64) NOT NULL DEFAULT 'civium-principal-000000000000000000000000000000000',
    ADD INDEX idx_web_invitations_network (network_id)
