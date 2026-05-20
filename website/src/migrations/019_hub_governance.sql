-- Migration 019 : gouvernance hub (propositions + votes)

CREATE TABLE IF NOT EXISTS hub_proposals (
    id          VARCHAR(36)  NOT NULL PRIMARY KEY,
    network_cid VARCHAR(64)  NOT NULL,
    author_cid  VARCHAR(64)  NOT NULL,
    title       VARCHAR(255) NOT NULL,
    description TEXT         NOT NULL,
    status      VARCHAR(16)  NOT NULL DEFAULT 'open',
    created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    closes_at   DATETIME     NULL,
    INDEX idx_hp_network (network_cid)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS hub_votes (
    id          VARCHAR(36) NOT NULL PRIMARY KEY,
    proposal_id VARCHAR(36) NOT NULL,
    network_cid VARCHAR(64) NOT NULL,
    voter_cid   VARCHAR(64) NOT NULL,
    choice      VARCHAR(8)  NOT NULL,
    voted_at    DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_voter_proposal (proposal_id, voter_cid),
    INDEX idx_hv_proposal (proposal_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
