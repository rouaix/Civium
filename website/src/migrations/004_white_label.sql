-- Migration 004 — White-label et licences par taille d'organisation
-- Permet à un opérateur de personnaliser son instance Civium (branding, limites).

CREATE TABLE IF NOT EXISTS white_label_settings (
  setting_key    VARCHAR(64)   NOT NULL,
  setting_value  TEXT          NOT NULL DEFAULT '',
  updated_at     DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (setting_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Valeurs par défaut de l'instance publique rouaix.com/civium
INSERT IGNORE INTO white_label_settings (setting_key, setting_value) VALUES
  ('app_name',       'Civium'),
  ('app_tagline',    'Votre réseau souverain'),
  ('app_logo_url',   ''),
  ('license_tier',   'open'),
  ('max_networks',   '0'),
  ('max_members',    '0'),
  ('contact_email',  ''),
  ('support_url',    ''),
  ('primary_color',  '#6366f1'),
  ('org_name',       ''),
  ('custom_css',     '');
