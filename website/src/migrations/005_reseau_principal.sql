-- Migration 005 : Réseau Principal Civium (web)
-- Tables pour le réseau hébergé côté serveur, accessible depuis le navigateur.
-- ENGINE=InnoDB explicite (default_storage_engine peut être MyISAM sur certains serveurs)

-- Membres du réseau principal
CREATE TABLE IF NOT EXISTS web_members (
    id           CHAR(36)     NOT NULL DEFAULT (UUID()),
    email        VARCHAR(191) NOT NULL UNIQUE,
    cid_short    VARCHAR(16)  DEFAULT NULL,
    cid_full     VARCHAR(64)  DEFAULT NULL,
    display_name VARCHAR(100) DEFAULT NULL,
    role         ENUM('admin','member') NOT NULL DEFAULT 'member',
    circle       TINYINT      NOT NULL DEFAULT 1,
    status       ENUM('pending','active','suspended') NOT NULL DEFAULT 'pending',
    invited_by   VARCHAR(16)  DEFAULT NULL,
    joined_at    DATETIME     DEFAULT NULL,
    created_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_web_members_email (email),
    INDEX idx_web_members_cid_short (cid_short)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Invitations envoyées par l'admin
CREATE TABLE IF NOT EXISTS web_invitations (
    id         CHAR(36)     NOT NULL DEFAULT (UUID()),
    token      CHAR(64)     NOT NULL UNIQUE,
    email      VARCHAR(191) NOT NULL,
    invited_by VARCHAR(16)  NOT NULL,
    expires_at DATETIME     NOT NULL,
    used       TINYINT(1)   NOT NULL DEFAULT 0,
    used_at    DATETIME     DEFAULT NULL,
    created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_web_invitations_token (token),
    INDEX idx_web_invitations_email (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Messages réseau (visibles par tous les membres)
CREATE TABLE IF NOT EXISTS web_messages (
    id           CHAR(36)     NOT NULL DEFAULT (UUID()),
    author_cid   VARCHAR(16)  NOT NULL,
    author_name  VARCHAR(100) NOT NULL,
    body         TEXT         NOT NULL,
    sent_at      DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_web_messages_sent_at (sent_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Messages directs entre deux membres
CREATE TABLE IF NOT EXISTS web_direct_messages (
    id         CHAR(36)    NOT NULL DEFAULT (UUID()),
    from_cid   VARCHAR(16) NOT NULL,
    to_cid     VARCHAR(16) NOT NULL,
    body       TEXT        NOT NULL,
    sent_at    DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_web_dm_from (from_cid),
    INDEX idx_web_dm_to (to_cid),
    INDEX idx_web_dm_sent_at (sent_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Propositions de gouvernance
CREATE TABLE IF NOT EXISTS web_proposals (
    id             CHAR(36)     NOT NULL DEFAULT (UUID()),
    title          VARCHAR(255) NOT NULL,
    description    TEXT         NOT NULL,
    options_json   TEXT         NOT NULL,
    created_by     VARCHAR(16)  NOT NULL,
    created_at     DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    closes_at      DATETIME     NOT NULL,
    quorum_percent TINYINT      NOT NULL DEFAULT 50,
    status         ENUM('open','closed','cancelled') NOT NULL DEFAULT 'open',
    PRIMARY KEY (id),
    INDEX idx_web_proposals_status (status),
    INDEX idx_web_proposals_closes_at (closes_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Votes sur les propositions
CREATE TABLE IF NOT EXISTS web_votes (
    id          CHAR(36)    NOT NULL DEFAULT (UUID()),
    proposal_id CHAR(36)    NOT NULL,
    voter_cid   VARCHAR(16) NOT NULL,
    option_idx  TINYINT     NOT NULL,
    cast_at     DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uq_web_votes_proposal_voter (proposal_id, voter_cid),
    INDEX idx_web_votes_proposal (proposal_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Entrées d'annuaire publiées
CREATE TABLE IF NOT EXISTS web_directory_entries (
    id           CHAR(36)                  NOT NULL DEFAULT (UUID()),
    kind         ENUM('network','member')  NOT NULL,
    subject_cid  VARCHAR(64)               NOT NULL,
    subject_name VARCHAR(191)              NOT NULL,
    description  TEXT                      DEFAULT NULL,
    tags         VARCHAR(500)              DEFAULT NULL,
    published_by VARCHAR(16)               NOT NULL,
    published_at DATETIME                  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_web_dir_kind (kind),
    INDEX idx_web_dir_subject_name (subject_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Fil d'activité du réseau
CREATE TABLE IF NOT EXISTS web_activity (
    id          CHAR(36)     NOT NULL DEFAULT (UUID()),
    kind        VARCHAR(50)  NOT NULL,
    actor_cid   VARCHAR(16)  NOT NULL,
    actor_name  VARCHAR(100) NOT NULL,
    summary     VARCHAR(500) NOT NULL,
    ref_id      CHAR(36)     DEFAULT NULL,
    occurred_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_web_activity_occurred_at (occurred_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Notifications par membre
CREATE TABLE IF NOT EXISTS web_notifications (
    id              CHAR(36)    NOT NULL DEFAULT (UUID()),
    member_cid      VARCHAR(16) NOT NULL,
    activity_id     CHAR(36)    NOT NULL,
    read_flag       TINYINT(1)  NOT NULL DEFAULT 0,
    created_at      DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_web_notif_member (member_cid),
    INDEX idx_web_notif_read (member_cid, read_flag)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
