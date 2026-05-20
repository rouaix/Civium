<?php
ini_set('display_errors', 1);
error_reporting(E_ALL);

$docRoot = is_dir(__DIR__ . '/src') ? __DIR__ . '/src' : dirname(__DIR__);

echo '<pre>';
echo '__DIR__       : ' . __DIR__   . "\n";
echo 'docRoot       : ' . $docRoot  . "\n";
echo 'PHP version   : ' . PHP_VERSION . "\n";
echo 'sodium        : ' . (function_exists('sodium_crypto_sign_detached') ? 'OK' : 'MANQUANT') . "\n";
echo 'bcmath        : ' . (function_exists('bcadd') ? 'OK' : 'MANQUANT') . "\n";
echo 'vendor        : ' . (file_exists($docRoot . '/vendor/autoload.php')  ? 'OK' : 'MANQUANT') . "\n";
echo 'config.ini    : ' . (file_exists($docRoot . '/app/config.ini')       ? 'OK' : 'MANQUANT') . "\n";
echo 'config.local  : ' . (file_exists($docRoot . '/app/config.local.ini') ? 'OK' : 'MANQUANT') . "\n";
echo 'tmp/          : ' . (is_dir($docRoot . '/tmp')      ? 'OK' : 'MANQUANT') . "\n";
echo 'tmp writable  : ' . (is_writable($docRoot . '/tmp') ? 'OK' : 'NON ACCESSIBLE') . "\n";
echo 'public/       : ' . (is_dir(__DIR__ . '/public')    ? 'OK' : 'MANQUANT') . "\n";

// Test chargement autoload
if (file_exists($docRoot . '/vendor/autoload.php')) {
    try {
        require_once $docRoot . '/vendor/autoload.php';
        echo 'autoload      : OK' . "\n";
    } catch (Throwable $e) {
        echo 'autoload ERR  : ' . $e->getMessage() . "\n";
    }
}

// Test config & BDD
if (file_exists($docRoot . '/app/config.local.ini')) {
    $ini = parse_ini_file($docRoot . '/app/config.local.ini', false, INI_SCANNER_RAW);
    $host = $ini['DB_HOST'] ?? null;
    $name = $ini['DB_NAME'] ?? null;
    $user = $ini['DB_USER'] ?? null;
    $pass = $ini['DB_PASS'] ?? null;
    echo 'DB_HOST       : ' . ($host ?? 'NON DÉFINI') . "\n";
    echo 'DB_NAME       : ' . ($name ?? 'NON DÉFINI') . "\n";
    echo 'DB_USER       : ' . ($user ?? 'NON DÉFINI') . "\n";
    echo 'DB_PASS       : ' . ($pass !== null ? '***' : 'NON DÉFINI') . "\n";

    try {
        $dsn = "mysql:host=$host;dbname=$name;charset=utf8mb4";
        $pdo = new PDO($dsn, $user, $pass);
        echo 'connexion BDD : OK' . "\n";
        $tables = $pdo->query("SHOW TABLES")->fetchAll(PDO::FETCH_COLUMN);
        echo 'tables        : ' . (count($tables) ? implode(', ', $tables) : 'AUCUNE') . "\n";
        $count = $pdo->query("SELECT COUNT(*) FROM waitlist")->fetchColumn();
        echo 'waitlist rows : ' . $count . "\n";
    } catch (Throwable $e) {
        echo 'BDD ERR       : ' . $e->getMessage() . "\n";
    }
}

echo '</pre>';
