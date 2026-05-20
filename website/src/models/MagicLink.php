<?php

/**
 * Gestion des tokens magic link pour l'authentification web.
 */
class MagicLink
{
    const EXPIRY_MINUTES = 15;

    /**
     * Crée un token pour l'email donné.
     * Retourne le token (hex 64 chars).
     */
    const RATE_LIMIT_PER_HOUR = 5;

    public static function create(\DB\SQL $db, string $email): string
    {
        // Nettoyage global des tokens expirés (tous emails confondus)
        $db->exec("DELETE FROM magic_links WHERE expires_at < NOW()");

        // Rate limit : max 5 tokens par email par heure
        $count = $db->exec(
            "SELECT COUNT(*) AS n FROM magic_links WHERE email = ? AND created_at >= DATE_SUB(NOW(), INTERVAL 1 HOUR)",
            [$email]
        );
        if ((int) ($count[0]['n'] ?? 0) >= self::RATE_LIMIT_PER_HOUR) {
            throw new \RuntimeException('rate_limited');
        }

        $token   = bin2hex(random_bytes(32));
        $expires = date('Y-m-d H:i:s', time() + self::EXPIRY_MINUTES * 60);

        $db->exec(
            "INSERT INTO magic_links (token, email, expires_at) VALUES (?, ?, ?)",
            [$token, $email, $expires]
        );

        return $token;
    }

    /**
     * Valide un token. Retourne l'email si valide, null sinon.
     * Marque le token comme utilisé au passage.
     */
    public static function validate(\DB\SQL $db, string $token): ?string
    {
        $rows = $db->exec(
            "SELECT email, used FROM magic_links WHERE token = ? AND expires_at > NOW() LIMIT 1",
            [$token]
        );

        if (!$rows || empty($rows[0])) return null;
        if ((int) $rows[0]['used'] === 1) return null;

        $email = $rows[0]['email'];
        $db->exec("UPDATE magic_links SET used = 1 WHERE token = ?", [$token]);

        return $email;
    }

    /**
     * Envoie (ou logue) le magic link.
     *
     * En mode DEBUG ou si pas de configuration SMTP, écrit dans data/magic_links.log.
     * En production, utilise PHP mail().
     */
    public static function send(Base $f3, string $email, string $token): void
    {
        // Dériver l'URL depuis APP_URL (authoritative) pour éviter tout double-préfixe
        $appUrl = rtrim((string) $f3->get('APP_URL'), '/');
        $url    = $appUrl . '/auth/verify?token=' . $token;
        $expiry = self::EXPIRY_MINUTES;

        $subject = 'Votre lien de connexion Civium';
        $body    = "Bonjour,\n\nCliquez sur ce lien pour vous connecter (valide $expiry minutes) :\n\n$url\n\nSi vous n'avez pas demandé ce lien, ignorez cet e-mail.\n\n— Civium";

        $debug   = (int) $f3->get('DEBUG') > 0;
        $logFile = __DIR__ . '/../data/magic_links.log';

        if ($debug) {
            // Mode développement : log du lien
            $line = date('[Y-m-d H:i:s]') . " email=$email token=$token url=$url\n";
            @file_put_contents($logFile, $line, FILE_APPEND);
            return;
        }

        $from     = (string) ($f3->get('MAIL_FROM')      ?: 'noreply@rouaix.com');
        $fromName = (string) ($f3->get('MAIL_FROM_NAME') ?: 'Civium');
        $smtpHost = (string) $f3->get('SMTP_HOST');

        if ($smtpHost) {
            // Envoi via SMTP (config.ini / config.prod.ini)
            $smtp = new \SMTP(
                $smtpHost,
                (int) ($f3->get('SMTP_PORT') ?: 587),
                (string) ($f3->get('SMTP_SCHEME') ?: 'tls'),
                (string) $f3->get('SMTP_USER'),
                (string) $f3->get('SMTP_PASS')
            );
            $smtp->set('From', "$fromName <$from>");
            $smtp->set('To', $email);
            $smtp->set('Subject', $subject);
            $smtp->set('Content-Type', 'text/plain; charset=UTF-8');
            $sent = $smtp->send($body);
            if (!$sent) {
                @file_put_contents($logFile, date('[Y-m-d H:i:s]') . " SMTP error email=$email\n", FILE_APPEND);
            }
        } else {
            // Fallback : mail() natif PHP
            $headers = "From: $fromName <$from>\r\nContent-Type: text/plain; charset=UTF-8";
            @mail($email, $subject, $body, $headers);
        }
    }
}
