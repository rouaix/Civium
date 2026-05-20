-- Identité Ed25519 du hub (une seule ligne)
CREATE TABLE IF NOT EXISTS hub_identity (
    id         INT         PRIMARY KEY DEFAULT 1,
    pubkey_b58 VARCHAR(64) NOT NULL,
    secret_b58 VARCHAR(64) NOT NULL,
    created_at DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Réseaux hébergés/relayés par ce hub
CREATE TABLE IF NOT EXISTS hub_networks (
    network_cid  VARCHAR(64)  PRIMARY KEY,
    network_name VARCHAR(128) NOT NULL,
    admin_cid    VARCHAR(64)  NOT NULL,
    admin_pubkey VARCHAR(64)  NOT NULL,
    created_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Membres enregistrés par réseau
CREATE TABLE IF NOT EXISTS hub_members (
    network_cid  VARCHAR(64)  NOT NULL,
    member_cid   VARCHAR(64)  NOT NULL,
    display_name VARCHAR(128) NOT NULL DEFAULT '',
    pubkey_b58   VARCHAR(64)  NOT NULL,
    joined_at    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (network_cid, member_cid),
    INDEX idx_hub_members_network (network_cid)
);

-- Messages relayés par le hub (payload chiffré côté client — hub ne voit pas le contenu)
CREATE TABLE IF NOT EXISTS hub_messages (
    id           BIGINT      AUTO_INCREMENT PRIMARY KEY,
    network_cid  VARCHAR(64) NOT NULL,
    message_id   VARCHAR(64) NOT NULL UNIQUE,
    sender_cid   VARCHAR(64) NOT NULL,
    payload_json MEDIUMTEXT  NOT NULL,
    received_at  DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_hub_messages_network (network_cid, received_at)
);
