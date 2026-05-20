-- Migration 002 — Fédération ActivityPub
-- Tables pour les acteurs ActivityPub, leurs abonnés, et leur boîte d'envoi.

-- Clés RSA et métadonnées de l'acteur AP pour chaque réseau activé
CREATE TABLE IF NOT EXISTS ap_actors (
  network_cid         VARCHAR(64)   NOT NULL,
  network_cid_short   VARCHAR(32)   NOT NULL,
  rsa_privkey         MEDIUMTEXT    NOT NULL,   -- PEM PKCS#8
  rsa_pubkey          TEXT          NOT NULL,   -- PEM public key
  created_at          DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (network_cid),
  KEY idx_cid_short (network_cid_short)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Abonnés ActivityPub (acteurs distants qui suivent un réseau Civium)
CREATE TABLE IF NOT EXISTS ap_followers (
  id              INT UNSIGNED  NOT NULL AUTO_INCREMENT,
  network_cid     VARCHAR(64)   NOT NULL,
  actor_url       TEXT          NOT NULL,
  inbox_url       TEXT          NOT NULL,
  shared_inbox    TEXT          DEFAULT NULL,
  followed_at     DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY uq_follow (network_cid, actor_url(512)),
  KEY idx_network (network_cid)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Boîte d'envoi : activités émises par les réseaux Civium
CREATE TABLE IF NOT EXISTS ap_outbox (
  id              INT UNSIGNED  NOT NULL AUTO_INCREMENT,
  network_cid     VARCHAR(64)   NOT NULL,
  activity_id     VARCHAR(512)  NOT NULL,
  activity_json   LONGTEXT      NOT NULL,
  delivered       TINYINT(1)    NOT NULL DEFAULT 0,
  created_at      DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY uq_activity_id (activity_id(512)),
  KEY idx_network_del (network_cid, delivered),
  KEY idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
