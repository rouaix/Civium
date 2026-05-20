-- Migration 017 : Corriger web_networks.id en VARCHAR(64)
-- La migration 006 a créé id en CHAR(36), insuffisant pour l'ID du réseau principal (50 chars)

ALTER TABLE web_networks
    MODIFY COLUMN id VARCHAR(64) NOT NULL
