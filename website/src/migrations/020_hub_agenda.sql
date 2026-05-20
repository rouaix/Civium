-- Migration 020 : agenda hub

CREATE TABLE IF NOT EXISTS hub_agenda_events (
    id          VARCHAR(36)  NOT NULL PRIMARY KEY,
    network_cid VARCHAR(64)  NOT NULL,
    author_cid  VARCHAR(64)  NOT NULL,
    title       VARCHAR(255) NOT NULL,
    description TEXT         NULL,
    start_at    DATETIME     NOT NULL,
    end_at      DATETIME     NULL,
    created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_hae_network (network_cid),
    INDEX idx_hae_start (start_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
