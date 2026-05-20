-- Migration 026 : messages directs hub (non chiffrés E2E — cercle 1-2)

CREATE TABLE IF NOT EXISTS hub_dms (
    id            VARCHAR(36)  NOT NULL PRIMARY KEY,
    network_cid   VARCHAR(64)  NOT NULL,
    sender_cid    VARCHAR(64)  NOT NULL,
    recipient_cid VARCHAR(64)  NOT NULL,
    sender_name   VARCHAR(128) NOT NULL DEFAULT '',
    body          TEXT         NOT NULL,
    sent_at       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_hdm_network (network_cid),
    INDEX idx_hdm_recipient (network_cid, recipient_cid, sent_at),
    INDEX idx_hdm_sender    (network_cid, sender_cid,    sent_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
