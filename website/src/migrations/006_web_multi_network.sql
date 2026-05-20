-- Migration 006 : Table des réseaux web (multi-réseau)

CREATE TABLE IF NOT EXISTS web_networks (
    id           CHAR(36)     NOT NULL,
    name         VARCHAR(191) NOT NULL,
    description  TEXT         DEFAULT NULL,
    admin_cid    VARCHAR(16)  NOT NULL,
    admin_email  VARCHAR(191) NOT NULL,
    is_public    TINYINT(1)   NOT NULL DEFAULT 0,
    created_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_web_networks_admin (admin_cid),
    INDEX idx_web_networks_public (is_public)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
