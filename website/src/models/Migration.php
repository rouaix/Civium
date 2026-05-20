<?php

class Migration
{
    public static function run(\DB\SQL $db): void
    {
        $db->exec("CREATE TABLE IF NOT EXISTS schema_migrations (
            version    INT          PRIMARY KEY,
            name       VARCHAR(191) NOT NULL,
            applied_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");

        $applied = array_column(
            $db->exec("SELECT version FROM schema_migrations ORDER BY version") ?: [],
            'version'
        );

        $files = glob(__DIR__ . '/../migrations/*.sql') ?: [];
        natsort($files);

        foreach ($files as $file) {
            if (!preg_match('/(\d+)_/', basename($file), $m)) continue;
            $version = (int) $m[1];
            if (in_array($version, $applied)) continue;

            $sql = file_get_contents($file);
            try {
                $db->begin();
                $db->exec($sql);
                $db->exec(
                    "INSERT INTO schema_migrations (version, name) VALUES (?, ?)",
                    [$version, basename($file)]
                );
                $db->commit();
                error_log("[Migration] Applied: " . basename($file));
            } catch (\Exception $e) {
                $db->rollback();
                error_log("[Migration] FAILED: " . basename($file) . " — " . $e->getMessage());
                http_response_code(503);
                die(json_encode([
                    'error'     => 'database_migration_failed',
                    'migration' => basename($file),
                    'message'   => $e->getMessage(),
                ]));
            }
        }
    }
}
