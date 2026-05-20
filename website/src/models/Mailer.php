<?php

/**
 * Envoi d'emails via F3 SMTP (ou PHP mail() en fallback).
 *
 * Config attendue dans config.ini / config.local.ini :
 *   SMTP_HOST   = smtp.example.com
 *   SMTP_PORT   = 587
 *   SMTP_SCHEME = tls          (ssl | tls | '')
 *   SMTP_USER   = user@example.com
 *   SMTP_PASS   = motdepasse
 *   MAIL_FROM   = noreply@rouaix.com
 *   MAIL_FROM_NAME = Civium RCC
 */
class Mailer
{
    /**
     * Envoie un email à tous les administrateurs enregistrés dans la table `networks`.
     * Appelé après l'enregistrement d'une alerte fraude.
     *
     * @param \DB\SQL $db
     * @param string  $alertType        Type court de l'alerte
     * @param string  $alertDescription Description complète
     * @param array   $networkCids      CIDs des réseaux concernés (peut être vide = toutes les admins)
     */
    public static function sendAlertToAdmins(
        \DB\SQL $db,
        string  $alertType,
        string  $alertDescription,
        array   $networkCids = []
    ): void {
        $f3 = Base::instance();

        // Récupère les emails des admins concernés (tous si $networkCids vide)
        if ($networkCids) {
            $placeholders = implode(',', array_fill(0, count($networkCids), '?'));
            $rows = $db->exec(
                "SELECT DISTINCT admin_email, network_name
                 FROM networks WHERE network_cid IN ({$placeholders})",
                array_values($networkCids)
            );
        } else {
            $rows = $db->exec(
                "SELECT DISTINCT admin_email, network_name FROM networks LIMIT 5000"
            );
        }

        if (!$rows) return;

        $subject  = '[Civium RCC] Alerte fraude : ' . $alertType;
        $from     = (string) $f3->get('MAIL_FROM')      ?: 'noreply@rouaix.com';
        $fromName = (string) $f3->get('MAIL_FROM_NAME') ?: 'Civium RCC';
        $appUrl   = rtrim((string) $f3->get('APP_URL'), '/');

        $htmlBody = self::buildAlertHtml($alertType, $alertDescription, $networkCids, $appUrl);
        $textBody = self::buildAlertText($alertType, $alertDescription, $networkCids, $appUrl);

        $smtpHost = (string) $f3->get('SMTP_HOST');

        foreach ($rows as $row) {
            $to = (string) $row['admin_email'];
            if (!filter_var($to, FILTER_VALIDATE_EMAIL)) continue;

            if ($smtpHost) {
                self::sendSmtp($f3, $from, $fromName, $to, $subject, $htmlBody, $textBody);
            } else {
                self::sendMail($from, $fromName, $to, $subject, $htmlBody, $textBody);
            }
        }
    }

    // ── Transports ────────────────────────────────────────────────────────────

    private static function sendSmtp(
        Base   $f3,
        string $from,
        string $fromName,
        string $to,
        string $subject,
        string $htmlBody,
        string $textBody
    ): void {
        try {
            $smtp = new \SMTP(
                (string) $f3->get('SMTP_HOST'),
                (int)    ($f3->get('SMTP_PORT') ?: 587),
                (string) ($f3->get('SMTP_SCHEME') ?: 'tls'),
                (string) $f3->get('SMTP_USER'),
                (string) $f3->get('SMTP_PASS')
            );
            $smtp->set('From',    '"' . addslashes($fromName) . '" <' . $from . '>');
            $smtp->set('To',      $to);
            $smtp->set('Subject', $subject);
            $smtp->set('Content-Type', 'text/html; charset=UTF-8');
            $smtp->send($htmlBody);
        } catch (\Exception $e) {
            error_log('[Mailer SMTP] ' . $to . ' — ' . $e->getMessage());
        }
    }

    private static function sendMail(
        string $from,
        string $fromName,
        string $to,
        string $subject,
        string $htmlBody,
        string $textBody
    ): void {
        $headers  = 'MIME-Version: 1.0' . "\r\n";
        $headers .= 'Content-Type: text/html; charset=UTF-8' . "\r\n";
        $headers .= 'From: "' . $fromName . '" <' . $from . '>' . "\r\n";
        $headers .= 'X-Mailer: Civium RCC' . "\r\n";
        @mail($to, $subject, $htmlBody, $headers);
    }

    /**
     * Email de bienvenue envoyé après création d'un compte avec mot de passe.
     */
    public static function sendWelcome(Base $f3, string $email, string $networkName): void
    {
        $scheme   = $f3->get('SCHEME') ?: 'https';
        $host     = $f3->get('HOST')   ?: parse_url((string) $f3->get('APP_URL'), PHP_URL_HOST) ?: 'localhost';
        $base     = rtrim((string) $f3->get('BASE'), '/');
        $appUrl   = $scheme . '://' . $host . $base;
        $from     = (string) $f3->get('MAIL_FROM')      ?: 'noreply@rouaix.com';
        $fromName = (string) $f3->get('MAIL_FROM_NAME') ?: 'Civium';

        $subject  = 'Bienvenue sur Civium — votre réseau est prêt';
        $htmlBody = self::buildWelcomeHtml($email, $networkName, $appUrl);
        $textBody = self::buildWelcomeText($email, $networkName, $appUrl);

        $smtpHost = (string) $f3->get('SMTP_HOST');
        if ($smtpHost) {
            self::sendSmtp($f3, $from, $fromName, $email, $subject, $htmlBody, $textBody);
        } else {
            self::sendMail($from, $fromName, $email, $subject, $htmlBody, $textBody);
        }
    }

    private static function buildWelcomeHtml(string $email, string $networkName, string $appUrl): string
    {
        $emailHtml   = htmlspecialchars($email, ENT_QUOTES);
        $networkHtml = htmlspecialchars($networkName, ENT_QUOTES);
        $loginUrl    = $appUrl . '/auth';
        $networkUrl  = $appUrl . '/civium/network';
        $desktopUrl  = 'https://github.com/rouaix/Civium/releases';

        return <<<HTML
        <!DOCTYPE html>
        <html lang="fr">
        <head><meta charset="UTF-8"><title>Bienvenue sur Civium</title></head>
        <body style="font-family:sans-serif;max-width:600px;margin:0 auto;padding:24px;color:#1e293b">
          <h1 style="color:#6366f1;margin-bottom:8px">Bienvenue sur Civium !</h1>
          <p>Votre compte a été créé avec l'adresse <strong>{$emailHtml}</strong>.</p>
          <p>Votre réseau <strong>«&nbsp;{$networkHtml}&nbsp;»</strong> est prêt. Vous pouvez dès maintenant :</p>
          <ul>
            <li><a href="{$networkUrl}">Accéder à votre réseau web</a></li>
            <li>Inviter des membres depuis le tableau de bord</li>
          </ul>
          <hr style="border:1px solid #e2e8f0;margin:24px 0">
          <h2 style="font-size:1rem;color:#334155">Connecter l'application desktop</h2>
          <p>Pour utiliser Civium sur votre ordinateur (application P2P complète) :</p>
          <ol>
            <li><a href="{$desktopUrl}">Télécharger l'application Civium Desktop</a></li>
            <li>Créer une identité dans l'app (section <em>Identité</em>)</li>
            <li>Rejoindre un réseau avec votre adresse e-mail : <code>{$emailHtml}</code></li>
          </ol>
          <hr style="border:1px solid #e2e8f0;margin:24px 0">
          <p style="font-size:13px;color:#94a3b8">
            Pour vous connecter : <a href="{$loginUrl}">{$loginUrl}</a><br>
            Ce message a été envoyé automatiquement par Civium.
          </p>
        </body>
        </html>
        HTML;
    }

    private static function buildWelcomeText(string $email, string $networkName, string $appUrl): string
    {
        return "Bienvenue sur Civium !\n\n"
             . "Votre compte : {$email}\n"
             . "Votre réseau : {$networkName}\n\n"
             . "Accéder à votre réseau : {$appUrl}/civium/network\n\n"
             . "Pour l'application desktop :\n"
             . "1. Télécharger : https://github.com/rouaix/Civium/releases\n"
             . "2. Créer une identité dans l'app\n"
             . "3. Rejoindre un réseau avec votre email : {$email}\n\n"
             . "Se connecter : {$appUrl}/auth\n\n"
             . "— Civium";
    }

    // ── Gabarit email ─────────────────────────────────────────────────────────

    private static function buildAlertHtml(
        string $type,
        string $description,
        array  $networkCids,
        string $appUrl
    ): string {
        $typeHtml  = htmlspecialchars($type, ENT_QUOTES);
        $descHtml  = nl2br(htmlspecialchars($description, ENT_QUOTES));
        $cidsHtml  = $networkCids
            ? '<ul>' . implode('', array_map(fn($c) => '<li><code>' . htmlspecialchars($c, ENT_QUOTES) . '</code></li>', $networkCids)) . '</ul>'
            : '<p><em>Tous les réseaux enregistrés</em></p>';

        return <<<HTML
        <!DOCTYPE html>
        <html lang="fr">
        <head><meta charset="UTF-8"><title>Alerte fraude Civium</title></head>
        <body style="font-family:sans-serif;max-width:600px;margin:0 auto;padding:24px;color:#1e293b">
          <h1 style="color:#dc2626;margin-bottom:8px">⚠ Alerte fraude RCC</h1>
          <p style="color:#64748b;margin-top:0">Registre Central Civium — notification automatique</p>
          <hr style="border:1px solid #e2e8f0;margin:16px 0">
          <p><strong>Type :</strong> {$typeHtml}</p>
          <p><strong>Description :</strong><br>{$descHtml}</p>
          <p><strong>Réseaux concernés :</strong></p>
          {$cidsHtml}
          <hr style="border:1px solid #e2e8f0;margin:16px 0">
          <p style="font-size:13px;color:#94a3b8">
            Ce message a été envoyé automatiquement par le Registre Central Civium.<br>
            <a href="{$appUrl}">Accéder au tableau de bord</a>
          </p>
        </body>
        </html>
        HTML;
    }

    private static function buildAlertText(
        string $type,
        string $description,
        array  $networkCids,
        string $appUrl
    ): string {
        $cids = $networkCids ? implode(', ', $networkCids) : 'tous les réseaux';
        return "ALERTE FRAUDE — Registre Central Civium\n\n"
             . "Type : {$type}\n\n"
             . "Description :\n{$description}\n\n"
             . "Réseaux concernés : {$cids}\n\n"
             . "— Civium RCC — {$appUrl}";
    }
}
