-- Migration 016 : Unicité (email, network_id) dans web_members au lieu d'email seul
-- "email" = index implicite créé par UNIQUE dans migration 005
-- "idx_web_members_email" = index explicite créé dans migration 005

ALTER TABLE web_members
    DROP INDEX email,
    DROP INDEX idx_web_members_email,
    ADD UNIQUE KEY uq_web_members_email_network (email, network_id)
