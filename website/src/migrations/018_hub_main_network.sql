-- Migration 018 : réseau principal Civium + colonne is_public sur hub_networks

-- Ajouter is_public aux réseaux hébergés
ALTER TABLE hub_networks
    ADD COLUMN is_public TINYINT(1) NOT NULL DEFAULT 0;

-- Table de configuration du réseau principal (une seule ligne, id=1)
CREATE TABLE IF NOT EXISTS hub_main_network (
    id           INT          PRIMARY KEY DEFAULT 1,
    network_cid  VARCHAR(64)  NOT NULL,
    network_name VARCHAR(128) NOT NULL DEFAULT 'Civium',
    secret_b58   VARCHAR(64)  NOT NULL,
    pubkey_b58   VARCHAR(64)  NOT NULL,
    created_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
