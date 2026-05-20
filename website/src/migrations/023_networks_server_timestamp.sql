-- Ajoute un horodatage serveur sur les enregistrements RCC (non falsifiable par le client)
-- Utilisé pour le rate limiting par IP côté serveur.
ALTER TABLE networks
    ADD COLUMN server_registered_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP;
