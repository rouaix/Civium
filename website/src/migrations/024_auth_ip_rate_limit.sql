-- Suivi des tentatives d'authentification par IP pour le rate limiting
CREATE TABLE IF NOT EXISTS auth_attempts (
    id         INT UNSIGNED     NOT NULL AUTO_INCREMENT,
    ip         VARCHAR(45)      NOT NULL,
    action     VARCHAR(32)      NOT NULL DEFAULT 'magic_link',
    created_at TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_auth_ip_action_ts (ip, action, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
