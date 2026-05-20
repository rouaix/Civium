<?php

/**
 * Endpoint de mise à jour Tauri v2.
 *
 * Format : GET /update/{target}/{arch}/{current_version}
 * Réponse 204 → pas de mise à jour disponible.
 * Réponse 200 → nouvelle version disponible (JSON signé Tauri).
 *
 * Le fichier update.json à la racine du dossier de données contient
 * la dernière version connue. S'il n'existe pas, retourne 204.
 */
class UpdateController extends \Controller {

    public function check(\Base $f3): void {
        $target  = $f3->get('PARAMS.target');
        $arch    = $f3->get('PARAMS.arch');
        $current = $f3->get('PARAMS.version');

        $dataFile = __DIR__ . '/../../../../data/update.json';

        if (!is_file($dataFile)) {
            http_response_code(204);
            return;
        }

        $data = json_decode(file_get_contents($dataFile), true);
        if (!$data || empty($data['version'])) {
            http_response_code(204);
            return;
        }

        // Compare semver: si current >= latest → pas de mise à jour
        if (version_compare($current, $data['version'], '>=')) {
            http_response_code(204);
            return;
        }

        $platform = $target . '-' . $arch;
        if (!isset($data['platforms'][$platform])) {
            http_response_code(204);
            return;
        }

        $p = $data['platforms'][$platform];
        header('Content-Type: application/json');
        echo json_encode([
            'version'  => $data['version'],
            'notes'    => $data['notes']    ?? '',
            'pub_date' => $data['pub_date'] ?? date('c'),
            'url'      => $p['url'],
            'signature'=> $p['signature'],
        ]);
    }
}
