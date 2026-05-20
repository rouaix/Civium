-- Comptes utilisateurs web avec mot de passe (alternative au magic link)
CREATE TABLE IF NOT EXISTS web_users (
    email         VARCHAR(191) NOT NULL PRIMARY KEY,
    password_hash VARCHAR(255) NOT NULL,
    created_at    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP
);
