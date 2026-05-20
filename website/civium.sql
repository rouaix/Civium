-- civium.sql : Bootstrap de la base de données
--
-- Ce fichier crée uniquement la base de données.
-- Le schéma complet (tables) est géré par le système de migrations :
--   website/src/migrations/001_initial.sql, 002_…, etc.
--
-- À exécuter UNE SEULE FOIS lors de l'installation initiale du serveur,
-- avant le premier lancement de l'application.
-- Les migrations s'appliquent ensuite automatiquement au démarrage PHP.

CREATE DATABASE IF NOT EXISTS civium
  DEFAULT CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci;
