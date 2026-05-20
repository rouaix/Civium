-- Migration 027 : notifications hub

CREATE TABLE IF NOT EXISTS hub_notifications (
    id           VARCHAR(36)  NOT NULL PRIMARY KEY,
    network_cid  VARCHAR(64)  NOT NULL,
    member_cid   VARCHAR(64)  NOT NULL,
    type         VARCHAR(32)  NOT NULL,
    title        VARCHAR(255) NOT NULL DEFAULT '',
    body         TEXT         NULL,
    is_read      TINYINT      NOT NULL DEFAULT 0,
    created_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_hn_member  (member_cid, is_read, created_at),
    INDEX idx_hn_network (network_cid, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
