-- Migration 001 — Schéma initial Civium
-- Contient le schéma issu de civium.sql (liste d'attente) + tables RCC.
-- La création de la base de données elle-même est dans civium.sql (à la racine).

-- Liste d'attente (site de présentation)
CREATE TABLE IF NOT EXISTS waitlist (
  id        INT UNSIGNED NOT NULL AUTO_INCREMENT,
  email     VARCHAR(255) NOT NULL,
  created   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY uq_email (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Registre Central Civium — réseaux enregistrés
CREATE TABLE IF NOT EXISTS networks (
  network_cid    VARCHAR(64)  NOT NULL,
  network_name   VARCHAR(255) NOT NULL,
  admin_cid      VARCHAR(64)  NOT NULL,
  admin_pubkey   VARCHAR(128) NOT NULL,
  admin_email    VARCHAR(255) NOT NULL,
  ip_address     VARCHAR(45)  NOT NULL,
  registered_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  signature      TEXT         NOT NULL,
  PRIMARY KEY (network_cid),
  KEY idx_admin_email (admin_email),
  KEY idx_registered_at (registered_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Alertes fraude émises par le RCC
CREATE TABLE IF NOT EXISTS alerts (
  id               INT UNSIGNED NOT NULL AUTO_INCREMENT,
  type             VARCHAR(64)  NOT NULL,
  description      TEXT         NOT NULL,
  network_cids     JSON         NOT NULL,
  emitted_at       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  emitted_by       VARCHAR(128) NOT NULL,
  PRIMARY KEY (id),
  KEY idx_emitted_at (emitted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Tokens magic link (authentification client web)
CREATE TABLE IF NOT EXISTS magic_links (
  token       VARCHAR(128) NOT NULL,
  email       VARCHAR(255) NOT NULL,
  cid         VARCHAR(64)  DEFAULT NULL,
  expires_at  DATETIME     NOT NULL,
  used        TINYINT(1)   NOT NULL DEFAULT 0,
  created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (token),
  KEY idx_email (email),
  KEY idx_expires_at (expires_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
