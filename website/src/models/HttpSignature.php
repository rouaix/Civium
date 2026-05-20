<?php

/**
 * Signatures HTTP pour ActivityPub (draft-cavage-http-signatures).
 *
 * Compatibilité Mastodon / PeerTube : RSA-SHA256, headers = (request-target) host date digest.
 */
class HttpSignature
{
    /**
     * Signe une requête HTTP sortante.
     *
     * @param string $method     Méthode HTTP en minuscules (post, get…)
     * @param string $url        URL complète de destination
     * @param string $body       Corps de la requête (peut être vide)
     * @param string $privkeyPem Clé privée RSA au format PEM
     * @param string $keyId      URI de la clé publique (ex. actor_url#main-key)
     * @return array  Headers à ajouter à la requête : Date, Digest, Signature
     */
    public static function sign(
        string $method,
        string $url,
        string $body,
        string $privkeyPem,
        string $keyId
    ): array {
        $parsed = parse_url($url);
        $host   = $parsed['host'];
        $path   = ($parsed['path'] ?? '/');
        if (!empty($parsed['query'])) {
            $path .= '?' . $parsed['query'];
        }

        $date    = gmdate('D, d M Y H:i:s') . ' GMT';
        $digest  = 'SHA-256=' . base64_encode(hash('sha256', $body, true));

        $headersToSign = "(request-target): " . strtolower($method) . " $path\n"
                       . "host: $host\n"
                       . "date: $date\n"
                       . "digest: $digest";

        $privkey = openssl_pkey_get_private($privkeyPem);
        if (!$privkey) {
            throw new \RuntimeException('Clé privée RSA invalide');
        }
        openssl_sign($headersToSign, $signature, $privkey, OPENSSL_ALGO_SHA256);

        return [
            'Date'      => $date,
            'Digest'    => $digest,
            'Signature' => sprintf(
                'keyId="%s",algorithm="rsa-sha256",headers="(request-target) host date digest",signature="%s"',
                $keyId,
                base64_encode($signature)
            ),
        ];
    }

    /**
     * Vérifie la signature HTTP d'une requête entrante.
     *
     * @param array  $headers Headers de la requête (clés en minuscules)
     * @param string $method  Méthode HTTP en minuscules
     * @param string $path    Chemin de la requête (avec query si présent)
     * @return string|null    URL de l'acteur distant si valide, null sinon
     */
    public static function verify(array $headers, string $method, string $path): ?string
    {
        $sigHeader = $headers['signature'] ?? '';
        if (!$sigHeader) return null;

        // Extraire les composants
        preg_match('/keyId="([^"]+)"/', $sigHeader, $m);
        $keyId = $m[1] ?? '';
        preg_match('/headers="([^"]+)"/', $sigHeader, $m);
        $headerNames = explode(' ', $m[1] ?? '(request-target) host date');
        preg_match('/signature="([^"]+)"/', $sigHeader, $m);
        $sigBytes = base64_decode($m[1] ?? '');

        if (!$keyId || !$sigBytes) return null;

        // Reconstruire la chaîne signée
        $lines = [];
        foreach ($headerNames as $h) {
            $h = trim($h);
            if ($h === '(request-target)') {
                $lines[] = "(request-target): " . strtolower($method) . " $path";
            } else {
                $val = $headers[strtolower($h)] ?? '';
                $lines[] = strtolower($h) . ": $val";
            }
        }
        $toVerify = implode("\n", $lines);

        // Récupérer la clé publique depuis le keyId (fetch acteur distant)
        $actorUrl = preg_replace('/#[^#]*$/', '', $keyId);
        $pubkeyPem = self::fetchPublicKey($keyId, $actorUrl);
        if (!$pubkeyPem) return null;

        $pubkey = openssl_pkey_get_public($pubkeyPem);
        if (!$pubkey) return null;

        $ok = openssl_verify($toVerify, $sigBytes, $pubkey, OPENSSL_ALGO_SHA256);
        return ($ok === 1) ? $actorUrl : null;
    }

    /**
     * Récupère la clé publique RSA d'un acteur AP distant.
     *
     * @param string $keyId    URI de la clé (ex. https://mastodon.social/users/alice#main-key)
     * @param string $actorUrl URL de l'acteur
     * @return string|null PEM public key ou null si erreur
     */
    public static function fetchPublicKey(string $keyId, string $actorUrl): ?string
    {
        $actor = self::fetchJson($actorUrl, [
            'Accept: application/activity+json, application/ld+json',
        ]);
        if (!$actor) return null;

        // Clé publique dans publicKey ou directement si le keyId pointe sur l'objet
        $pk = $actor['publicKey'] ?? null;
        if (!$pk && isset($actor['id']) && $actor['id'] === $keyId) {
            $pk = $actor;
        }
        if (!is_array($pk)) return null;

        return $pk['publicKeyPem'] ?? null;
    }

    /**
     * Envoie une activité ActivityPub signée à une boîte de réception distante.
     *
     * @param string $inboxUrl     URL inbox destinataire
     * @param string $activityJson Corps JSON de l'activité
     * @param string $privkeyPem   Clé privée RSA de l'acteur émetteur
     * @param string $keyId        URI de la clé publique (actor_url#main-key)
     * @return bool Succès ou échec
     */
    public static function deliver(
        string $inboxUrl,
        string $activityJson,
        string $privkeyPem,
        string $keyId
    ): bool {
        try {
            $signedHeaders = self::sign('post', $inboxUrl, $activityJson, $privkeyPem, $keyId);
        } catch (\Exception $e) {
            error_log("[AP deliver] Sign error: " . $e->getMessage());
            return false;
        }

        $ch = curl_init($inboxUrl);
        curl_setopt_array($ch, [
            CURLOPT_POST           => true,
            CURLOPT_POSTFIELDS     => $activityJson,
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT        => 10,
            CURLOPT_HTTPHEADER     => [
                'Content-Type: application/activity+json',
                'Date: ' . $signedHeaders['Date'],
                'Digest: ' . $signedHeaders['Digest'],
                'Signature: ' . $signedHeaders['Signature'],
                'User-Agent: Civium/1.0 (ActivityPub)',
            ],
        ]);
        $response = curl_exec($ch);
        $code     = (int) curl_getinfo($ch, CURLINFO_HTTP_CODE);
        curl_close($ch);

        if ($code < 200 || $code >= 300) {
            error_log("[AP deliver] HTTP $code for $inboxUrl — " . substr($response, 0, 200));
            return false;
        }
        return true;
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    public static function fetchJson(string $url, array $extraHeaders = []): ?array
    {
        $ch = curl_init($url);
        curl_setopt_array($ch, [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT        => 10,
            CURLOPT_FOLLOWLOCATION => true,
            CURLOPT_MAXREDIRS      => 3,
            CURLOPT_HTTPHEADER     => array_merge([
                'Accept: application/activity+json, application/ld+json; profile="https://www.w3.org/ns/activitystreams"',
                'User-Agent: Civium/1.0 (ActivityPub)',
            ], $extraHeaders),
        ]);
        $body = curl_exec($ch);
        $code = (int) curl_getinfo($ch, CURLINFO_HTTP_CODE);
        curl_close($ch);

        if ($code < 200 || $code >= 300 || !$body) return null;
        $data = json_decode($body, true);
        return is_array($data) ? $data : null;
    }

    /**
     * Normalise les headers PHP ($_SERVER) en tableau clé-minuscule → valeur.
     */
    public static function requestHeaders(): array
    {
        $headers = [];
        foreach ($_SERVER as $key => $val) {
            if (str_starts_with($key, 'HTTP_')) {
                $name = strtolower(str_replace('_', '-', substr($key, 5)));
                $headers[$name] = $val;
            }
        }
        // CONTENT_TYPE et CONTENT_LENGTH ne sont pas préfixés HTTP_
        if (isset($_SERVER['CONTENT_TYPE'])) {
            $headers['content-type'] = $_SERVER['CONTENT_TYPE'];
        }
        return $headers;
    }
}
