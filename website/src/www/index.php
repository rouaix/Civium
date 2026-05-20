<?php

$logFile = (is_dir(__DIR__ . '/src') ? __DIR__ . '/src' : dirname(__DIR__)) . '/tmp/php_error.log';

set_error_handler(function ($errno, $errstr, $errfile, $errline) use ($logFile) {
    file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "ERROR $errno: $errstr in $errfile:$errline\n", FILE_APPEND);
});

set_exception_handler(function (Throwable $e) use ($logFile) {
    file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "EXCEPTION: " . $e->getMessage() . " in " . $e->getFile() . ":" . $e->getLine() . "\n" . $e->getTraceAsString() . "\n", FILE_APPEND);
    http_response_code(500);
    echo '<pre>EXCEPTION: ' . htmlspecialchars($e->getMessage()) . '</pre>';
    exit;
});

register_shutdown_function(function () use ($logFile) {
    $error = error_get_last();
    if ($error && in_array($error['type'], [E_ERROR, E_PARSE, E_CORE_ERROR, E_COMPILE_ERROR])) {
        file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "FATAL: {$error['message']} in {$error['file']}:{$error['line']}\n", FILE_APPEND);
        echo '<pre>FATAL: ' . htmlspecialchars($error['message']) . '</pre>';
    }
});

file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "index.php started, __DIR__=" . __DIR__ . "\n", FILE_APPEND);

// Security headers
header('X-Content-Type-Options: nosniff');
header('X-Frame-Options: DENY');
header('Referrer-Policy: strict-origin-when-cross-origin');
header('Permissions-Policy: geolocation=(), microphone=(), camera=()');
header('Strict-Transport-Security: max-age=31536000; includeSubDomains');
header(
    "Content-Security-Policy: " .
    "default-src 'self'; " .
    "script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://esm.sh 'wasm-unsafe-eval'; " .
    "style-src 'self' 'unsafe-inline'; " .
    "img-src 'self' data: blob:; " .
    "connect-src 'self' wss: ws: https://esm.sh; " .
    "font-src 'self'; " .
    "object-src 'none'; " .
    "base-uri 'self'; " .
    "frame-ancestors 'none'; " .
    "form-action 'self'"
);

$docRoot = is_dir(__DIR__ . '/src') ? __DIR__ . '/src' : dirname(__DIR__);
file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "docRoot=$docRoot\n", FILE_APPEND);

chdir($docRoot);
require_once $docRoot . '/vendor/autoload.php';
file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "autoload OK\n", FILE_APPEND);

$f3 = Base::instance();
$f3->set('DEBUG', 3);
$f3->set('ROOT', $docRoot);

$f3->config($docRoot . '/app/config.ini');
file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "config.ini OK\n", FILE_APPEND);

// En local (chemin contient "Projets") on charge config.local.ini ; en prod, config.prod.ini.
$isLocal = strpos(__DIR__, 'Projets') !== false;
$envConfig = $isLocal
    ? $docRoot . '/app/config.local.ini'
    : $docRoot . '/app/config.prod.ini';
if (is_file($envConfig)) {
    try {
        $f3->config($envConfig);
        file_put_contents($logFile, date('[Y-m-d H:i:s] ') . basename($envConfig) . " OK\n", FILE_APPEND);
    } catch (\Exception $e) {
        file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "Config error (" . basename($envConfig) . "): " . $e->getMessage() . "\n", FILE_APPEND);
    }
}

$f3->set('ONERROR', function (Base $f3) {
    $code = (int) $f3->get('ERROR.code');
    $text = (string) $f3->get('ERROR.text');
    http_response_code($code);
    echo '<!DOCTYPE html><html lang="fr"><body><h1>' . $code . '</h1><p>' . htmlspecialchars($text) . '</p></body></html>';
    exit;
});

// Bootstrap DB + migrations (seulement si DB configurée)
if ($f3->get('DB_HOST') && $f3->get('DB_NAME')) {
    try {
        $dsn = 'mysql:host=' . $f3->get('DB_HOST')
             . ';dbname=' . $f3->get('DB_NAME')
             . ';charset=utf8mb4';
        $db = new \DB\SQL(
            $dsn,
            (string) $f3->get('DB_USER'),
            (string) $f3->get('DB_PASS')
        );
        $f3->set('DB', $db);
        file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "DB connected\n", FILE_APPEND);
        Migration::run($db);
        file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "Migrations OK\n", FILE_APPEND);

        // Bootstrap du réseau principal et du compte admin.
        $principalNetworkId = 'civium-principal-000000000000000000000000000000000';
        $adminEmail = (string) $f3->get('ADMIN_EMAIL');
        if ($adminEmail) {
            // Crée le réseau principal s'il n'existe pas encore
            $netExists = $db->exec(
                "SELECT COUNT(*) AS n FROM web_networks WHERE id = ?",
                [$principalNetworkId]
            );
            if ((int)($netExists[0]['n'] ?? 0) === 0) {
                $db->exec(
                    "INSERT IGNORE INTO web_networks (id, name, admin_cid, admin_email, is_public) VALUES (?, 'Réseau Civium', 'admin', ?, 0)",
                    [$principalNetworkId, $adminEmail]
                );
                file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "Principal network created\n", FILE_APPEND);
            }
            // Crée le compte admin si aucun n'existe encore dans ce réseau
            $existing = $db->exec(
                "SELECT COUNT(*) AS n FROM web_members WHERE role = 'admin' AND network_id = ?",
                [$principalNetworkId]
            );
            if ((int)($existing[0]['n'] ?? 0) === 0) {
                $db->exec(
                    "INSERT IGNORE INTO web_members (network_id, email, role, status, display_name) VALUES (?, ?, 'admin', 'pending', ?)",
                    [$principalNetworkId, $adminEmail, (string) $f3->get('ADMIN_USER') ?: 'Admin']
                );
                file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "Admin web bootstrapped: $adminEmail\n", FILE_APPEND);
            }
            // Enregistre le réseau principal dans la table RCC (networks) s'il n'y est pas encore.
            // Champs cryptographiques non applicables au client web → valeurs indicatives.
            $rccExists = $db->exec(
                "SELECT COUNT(*) AS n FROM networks WHERE network_cid = ?",
                [$principalNetworkId]
            );
            if ((int)($rccExists[0]['n'] ?? 0) === 0) {
                $ip = $_SERVER['SERVER_ADDR'] ?? '127.0.0.1';
                $db->exec(
                    "INSERT IGNORE INTO networks
                        (network_cid, network_name, admin_cid, admin_pubkey, admin_email, ip_address, signature)
                     VALUES (?, 'Réseau Civium (principal web)', 'web-admin', 'web-client', ?, ?, 'web-client')",
                    [$principalNetworkId, $adminEmail, $ip]
                );
                file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "Principal network registered in RCC table\n", FILE_APPEND);
            }
        }
    } catch (\Exception $e) {
        file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "DB/Migration error: " . $e->getMessage() . "\n", FILE_APPEND);
        // Continuer sans DB — les routes API retourneront une erreur si elles en ont besoin
    }
}

// Force F3 BASE depuis SCRIPT_NAME (plus fiable qu'APP_URL avec cPanel/symlinks).
// SCRIPT_NAME = /civium/index.php → BASE = /civium
// SCRIPT_NAME = /index.php       → dirname = '/' → BASE vide ('')
// IMPORTANT : F3 peut auto-setter BASE à '/' en racine, ce qui donnerait
// href="//public/..." (URL protocol-relative) → on force '' dans ce cas.
$_scriptDir = rtrim(dirname($_SERVER['SCRIPT_NAME'] ?? ''), '/');
if ($_scriptDir === '.' || $_scriptDir === '/' || $_scriptDir === '') {
    $_scriptDir = '';
}
$f3->set('BASE', $_scriptDir);

file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "calling f3->run()\n", FILE_APPEND);
$f3->run();
file_put_contents($logFile, date('[Y-m-d H:i:s] ') . "f3->run() done\n", FILE_APPEND);
