-- Migration 025 : documents collaboratifs hub

CREATE TABLE IF NOT EXISTS hub_documents (
    id             VARCHAR(36)   NOT NULL PRIMARY KEY,
    network_cid    VARCHAR(64)   NOT NULL,
    author_cid     VARCHAR(64)   NOT NULL,
    title          VARCHAR(255)  NOT NULL,
    content        MEDIUMTEXT    NOT NULL DEFAULT '',
    last_edited_by VARCHAR(64)   NOT NULL,
    created_at     DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_hdoc_network (network_cid),
    INDEX idx_hdoc_updated (updated_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
