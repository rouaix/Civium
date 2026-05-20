-- Migration 013 : network_id dans web_directory_entries

ALTER TABLE web_directory_entries
    ADD COLUMN network_id VARCHAR(64) NOT NULL DEFAULT 'civium-principal-000000000000000000000000000000000',
    ADD INDEX idx_web_dir_network (network_id)
