-- Migration 021 : hiérarchie réseau-de-réseaux (parent_network_cid)

ALTER TABLE hub_networks
    ADD COLUMN parent_network_cid VARCHAR(64) NULL DEFAULT NULL;
