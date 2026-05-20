import { useState } from "react";
import { tauriInvoke } from "../tauri";
import type { IdentityInfo, NetworkInfo } from "../types";

const MAX_TEXT = 256;
const B58_ALPHABET = /^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]+$/;
const NULL_BYTE = /\0/;
const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function validateName(v: string): string | null {
  const s = v.trim();
  if (!s) return "Ce champ est requis.";
  if (s.length > MAX_TEXT) return `Maximum ${MAX_TEXT} caractères.`;
  if (NULL_BYTE.test(s)) return "Caractère non autorisé.";
  return null;
}

function validateEmail(v: string): string | null {
  const s = v.trim();
  if (!s) return "L'adresse email est requise.";
  if (s.length > MAX_TEXT) return `Maximum ${MAX_TEXT} caractères.`;
  if (!EMAIL_RE.test(s)) return "Format d'adresse email invalide.";
  return null;
}

function validateB58(v: string): string | null {
  const s = v.trim();
  if (!s) return "La clé secrète est requise.";
  if (s.length < 32 || s.length > 512) return "Longueur de clé invalide (32–512 caractères attendus).";
  if (!B58_ALPHABET.test(s)) return "La clé contient des caractères non Base58.";
  return null;
}

interface RestoreFromBackupProps {
  onSuccess: (id: IdentityInfo) => void;
  onError: (msg: string) => void;
}

function RestoreFromBackup({ onSuccess, onError }: RestoreFromBackupProps) {
  const [password, setPassword] = useState("");
  const [fileName, setFileName] = useState<string | null>(null);
  const [fileB64, setFileB64] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  function handleFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setFileName(file.name);
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      // result = "data:<mime>;base64,<data>"
      const b64 = result.split(",")[1] ?? "";
      setFileB64(b64);
    };
    reader.readAsDataURL(file);
  }

  async function restore() {
    if (!fileB64) { onError("Aucun fichier sélectionné."); return; }
    if (!password) { onError("Le mot de passe est requis."); return; }
    setLoading(true);
    try {
      const id = await tauriInvoke<IdentityInfo>("identity_backup_import", {
        backupB64: fileB64,
        password,
      });
      onSuccess(id);
    } catch (e) {
      onError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-2">
      <label className="block">
        <span className="sr-only">Choisir un fichier .civium-backup</span>
        <input
          type="file"
          accept=".civium-backup"
          onChange={handleFile}
          className="block w-full text-xs text-gray-600
                     file:mr-3 file:py-1.5 file:px-3
                     file:rounded-lg file:border-0
                     file:text-xs file:font-medium
                     file:bg-amber-100 file:text-amber-800
                     hover:file:bg-amber-200 cursor-pointer"
        />
      </label>
      {fileName && (
        <p className="text-xs text-amber-700 font-mono truncate">{fileName}</p>
      )}
      <input
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        onKeyDown={(e) => { if (e.key === "Enter") restore(); }}
        placeholder="Mot de passe du fichier de sauvegarde"
        className="w-full border border-amber-200 bg-white rounded-lg px-3 py-2 text-sm
                   focus:outline-none focus:ring-2 focus:ring-amber-400"
      />
      <button
        onClick={restore}
        disabled={loading || !fileB64 || !password}
        className="w-full py-2 bg-amber-600 text-white rounded-lg text-sm font-medium
                   hover:bg-amber-700 disabled:opacity-50 transition-colors"
      >
        {loading ? "Déchiffrement…" : "Restaurer depuis ce fichier"}
      </button>
    </div>
  );
}

type Step = "welcome" | "identity" | "restore" | "email" | "choice" | "create" | "join" | "done";

const STEP_NUMBER: Record<Step, number> = {
  welcome:  0,
  identity: 1,
  restore:  1,
  email:    2,
  choice:   3,
  create:   4,
  join:     4,
  done:     5,
};
const TOTAL_STEPS = 5;
type Mode = "create" | "join";

interface Props {
  onComplete: () => void;
}

export default function Onboarding({ onComplete }: Props) {
  const [step, setStep] = useState<Step>("welcome");
  const [mode, setMode] = useState<Mode>("create");
  const [identity, setIdentity] = useState<IdentityInfo | null>(null);
  const [network, setNetwork] = useState<NetworkInfo | null>(null);
  const [adminEmail, setAdminEmail] = useState("");
  const [networkName, setNetworkName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [inviteLink, setInviteLink] = useState<string | null>(null);
  const [joinInviteLink, setJoinInviteLink] = useState("");
  const [peerAddr, setPeerAddr] = useState("");
  const [restoreSecret, setRestoreSecret] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function createIdentity() {
    setLoading(true);
    setError(null);
    try {
      const id = await tauriInvoke<IdentityInfo>("identity_init");
      setIdentity(id);
      setStep("email");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function restoreIdentity() {
    const err = validateB58(restoreSecret);
    if (err) { setError(err); return; }
    setLoading(true);
    setError(null);
    try {
      const id = await tauriInvoke<IdentityInfo>("identity_restore_from_secret", {
        secretB58: restoreSecret.trim(),
      });
      setIdentity(id);
      setStep("email");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function saveEmail() {
    const err = validateEmail(adminEmail);
    if (err) { setError(err); return; }
    setLoading(true);
    setError(null);
    try {
      await tauriInvoke("profile_email_set", { email: adminEmail.trim() });
      setStep("choice");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function createNetwork() {
    const errName = validateName(networkName) ?? validateName(displayName);
    if (errName) { setError(errName); return; }
    setLoading(true);
    setError(null);
    try {
      const net = await tauriInvoke<NetworkInfo>("network_create", {
        name: networkName.trim(),
        displayName: displayName.trim(),
        privacy: false,
      });
      setNetwork(net);

      // Enregistrement automatique au RCC
      if (adminEmail.trim()) {
        tauriInvoke("rcc_register", {
          networkCid: net.cid_short,
          adminEmail: adminEmail.trim(),
        }).catch(() => {});
      }

      const link = await tauriInvoke<string>("network_invite", {
        networkCid: net.cid_short,
        expiresIn: 0,
      });
      setInviteLink(link);
      setStep("done");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function joinNetwork() {
    const errDisplay = validateName(displayName);
    if (errDisplay) { setError(errDisplay); return; }
    if (!joinInviteLink.trim()) { setError("Le lien d'invitation est requis."); return; }
    if (joinInviteLink.trim().length > 2048) { setError("Lien d'invitation trop long."); return; }
    setLoading(true);
    setError(null);
    try {
      let net: NetworkInfo;
      if (peerAddr.trim()) {
        net = await tauriInvoke<NetworkInfo>("network_join_p2p", {
          inviteLink: joinInviteLink.trim(),
          displayName: displayName.trim(),
          peerAddr: peerAddr.trim(),
        });
      } else {
        net = await tauriInvoke<NetworkInfo>("network_join", {
          inviteLink: joinInviteLink.trim(),
          displayName: displayName.trim(),
        });
      }
      setNetwork(net);
      setStep("done");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  function pickMode(m: Mode) {
    setMode(m);
    setDisplayName("");
    setError(null);
    setStep(m === "create" ? "create" : "join");
  }

  const joinLoadingLabel = peerAddr.trim()
    ? "Connexion P2P… (jusqu'à 30 s)"
    : "Rejoindre…";

  const stepNum = STEP_NUMBER[step];

  return (
    <main className="min-h-screen flex items-center justify-center bg-gradient-to-br from-civium-50 to-civium-100 p-6" aria-label="Configuration initiale de Civium">
      <div className="bg-white rounded-2xl shadow-lg max-w-md w-full p-8">

        {/* Progress indicator — hidden on welcome and done */}
        {stepNum > 0 && stepNum < TOTAL_STEPS && (
          <div className="mb-6">
            <div className="flex items-center justify-between mb-1.5">
              <span className="text-xs text-gray-400">Configuration</span>
              <span className="text-xs font-medium text-civium-600">{stepNum} / {TOTAL_STEPS}</span>
            </div>
            <div className="w-full bg-gray-100 rounded-full h-1.5">
              <div
                className="bg-civium-600 h-1.5 rounded-full transition-all duration-300"
                style={{ width: `${(stepNum / TOTAL_STEPS) * 100}%` }}
              />
            </div>
          </div>
        )}

        {/* Welcome */}
        {step === "welcome" && (
          <div className="text-center space-y-6">
            <div className="text-5xl">🌐</div>
            <h1 className="text-2xl font-bold text-civium-900">
              Bienvenue sur Civium
            </h1>
            <p className="text-gray-600 text-sm leading-relaxed">
              Civium vous permet de créer des espaces souverains pour votre
              famille, association ou équipe — sans serveur central.
            </p>
            <button
              onClick={() => setStep("identity")}
              className="w-full py-3 bg-civium-600 text-white rounded-xl font-semibold
                         hover:bg-civium-700 transition-colors"
            >
              Créer une nouvelle identité
            </button>
            <button
              onClick={() => setStep("restore")}
              className="w-full py-2 text-sm text-civium-600 hover:text-civium-800 transition-colors"
            >
              J'ai déjà un compte — restaurer mon identité
            </button>
          </div>
        )}

        {/* Restore identity */}
        {step === "restore" && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-bold text-gray-900">Restaurer mon identité</h2>
              <p className="text-sm text-gray-500 mt-1">
                Saisissez votre clé secrète ou importez un fichier de sauvegarde chiffré.
              </p>
            </div>

            {/* Option 1 : fichier de backup chiffré */}
            <div className="bg-amber-50 border border-amber-200 rounded-xl p-4 space-y-3">
              <p className="text-sm font-medium text-amber-900">Option 1 — Fichier de sauvegarde chiffré (.civium-backup)</p>
              <RestoreFromBackup onSuccess={(id) => { setIdentity(id); setStep("email"); }} onError={setError} />
            </div>

            {/* Option 2 : clé secrète brute */}
            <div className="space-y-2">
              <p className="text-sm font-medium text-gray-700">Option 2 — Clé secrète (secret_b58)</p>
              <textarea
                value={restoreSecret}
                onChange={(e) => setRestoreSecret(e.target.value)}
                placeholder="Collez ici votre clé secrète (secret_b58)…"
                rows={3}
                maxLength={512}
                className="w-full border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono
                           focus:outline-none focus:ring-2 focus:ring-civium-500 resize-none"
              />
              <p className="text-xs text-gray-400">
                Disponible dans Paramètres → Identité → Clé secrète de votre autre appareil.
              </p>
            </div>

            {error && (
              <div role="alert" aria-live="assertive" className="bg-red-50 text-red-700 text-sm rounded-lg px-4 py-3">{error}</div>
            )}
            <div className="flex gap-3">
              <button
                onClick={() => { setStep("welcome"); setError(null); }}
                className="px-4 py-3 text-sm text-gray-500 hover:text-gray-700 transition-colors"
              >
                ← Retour
              </button>
              <button
                onClick={restoreIdentity}
                disabled={loading || !restoreSecret.trim()}
                className="flex-1 py-3 bg-civium-600 text-white rounded-xl font-semibold
                           hover:bg-civium-700 disabled:opacity-50 transition-colors"
              >
                {loading ? "Restauration…" : "Restaurer depuis la clé secrète"}
              </button>
            </div>
          </div>
        )}

        {/* Create identity */}
        {step === "identity" && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-bold text-gray-900">
                Créer votre identité
              </h2>
              <p className="text-sm text-gray-500 mt-1">
                Une paire de clés Ed25519 sera générée localement. Votre CID est
                dérivé de votre clé publique — unique par garantie cryptographique.
              </p>
            </div>
            {error && (
              <div className="bg-red-50 text-red-700 text-sm rounded-lg px-4 py-3">
                {error}
              </div>
            )}
            <button
              onClick={createIdentity}
              disabled={loading}
              aria-busy={loading}
              className="w-full py-3 bg-civium-600 text-white rounded-xl font-semibold
                         hover:bg-civium-700 disabled:opacity-50 transition-colors"
            >
              {loading ? "Génération…" : "Générer mon identité"}
            </button>
          </div>
        )}

        {/* Email */}
        {step === "email" && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-bold text-gray-900">Votre email de contact</h2>
              <p className="text-sm text-gray-500 mt-1">
                Requis pour enregistrer vos réseaux au Registre Central Civium (RCC — registre légal).
                Il ne sera jamais partagé publiquement.
              </p>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Adresse email <span className="text-red-500">*</span>
              </label>
              <input
                type="email"
                value={adminEmail}
                onChange={(e) => setAdminEmail(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") saveEmail(); }}
                placeholder="votre@email.com"
                maxLength={MAX_TEXT}
                className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm
                           focus:outline-none focus:ring-2 focus:ring-civium-500"
                autoFocus
              />
            </div>
            {error && (
              <div className="bg-red-50 text-red-700 text-sm rounded-lg px-4 py-3">{error}</div>
            )}
            <button
              onClick={saveEmail}
              disabled={loading || !adminEmail.trim()}
              className="w-full py-3 bg-civium-600 text-white rounded-xl font-semibold
                         hover:bg-civium-700 disabled:opacity-50 transition-colors"
            >
              {loading ? "Enregistrement…" : "Continuer"}
            </button>
          </div>
        )}

        {/* Choice: create or join */}
        {step === "choice" && identity && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-bold text-gray-900">
                Votre identité est prête
              </h2>
              <p className="text-sm text-gray-500 mt-1">
                CID : <code className="font-mono text-xs bg-gray-100 px-1 rounded">
                  {identity.cid_short}
                </code>
              </p>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <button
                onClick={() => pickMode("create")}
                className="flex flex-col items-center gap-2 p-4 border-2 border-civium-200
                           rounded-xl hover:border-civium-500 hover:bg-civium-50 transition-colors"
              >
                <span className="text-2xl">✨</span>
                <span className="text-sm font-semibold text-gray-800">Créer un réseau</span>
                <span className="text-xs text-gray-500 text-center">Nouveau groupe, vous êtes admin</span>
              </button>
              <button
                onClick={() => pickMode("join")}
                className="flex flex-col items-center gap-2 p-4 border-2 border-civium-200
                           rounded-xl hover:border-civium-500 hover:bg-civium-50 transition-colors"
              >
                <span className="text-2xl">🔗</span>
                <span className="text-sm font-semibold text-gray-800">Rejoindre</span>
                <span className="text-xs text-gray-500 text-center">Via un lien d'invitation</span>
              </button>
            </div>
          </div>
        )}

        {/* Create network */}
        {step === "create" && identity && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-bold text-gray-900">
                Créer votre premier réseau
              </h2>
              <p className="text-sm text-gray-500 mt-1">
                Votre CID : <code className="font-mono text-xs bg-gray-100 px-1 rounded">
                  {identity.cid_short}
                </code>
              </p>
            </div>
            <div className="space-y-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Nom du réseau
                </label>
                <input
                  type="text"
                  value={networkName}
                  onChange={(e) => setNetworkName(e.target.value)}
                  placeholder="ex. Famille Dupont, Asso Voisins…"
                  maxLength={MAX_TEXT}
                  className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm
                             focus:outline-none focus:ring-2 focus:ring-civium-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Votre nom dans ce réseau
                </label>
                <input
                  type="text"
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  placeholder="ex. Marie, Admin…"
                  maxLength={MAX_TEXT}
                  className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm
                             focus:outline-none focus:ring-2 focus:ring-civium-500"
                />
              </div>
            </div>
            {error && (
              <div className="bg-red-50 text-red-700 text-sm rounded-lg px-4 py-3">
                {error}
              </div>
            )}
            <div className="flex gap-3">
              <button
                onClick={() => setStep("choice")}
                className="px-4 py-3 text-sm text-gray-500 hover:text-gray-700 transition-colors"
              >
                ← Retour
              </button>
              <button
                onClick={createNetwork}
                disabled={loading || !networkName.trim() || !displayName.trim()}
                className="flex-1 py-3 bg-civium-600 text-white rounded-xl font-semibold
                           hover:bg-civium-700 disabled:opacity-50 transition-colors"
              >
                {loading ? "Création…" : "Créer le réseau"}
              </button>
            </div>
          </div>
        )}

        {/* Join network */}
        {step === "join" && identity && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-bold text-gray-900">
                Rejoindre un réseau
              </h2>
              <p className="text-sm text-gray-500 mt-1">
                Collez le lien d'invitation et l'adresse d'un pair actif.
              </p>
            </div>
            <div className="space-y-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Lien d'invitation
                </label>
                <textarea
                  value={joinInviteLink}
                  onChange={(e) => setJoinInviteLink(e.target.value)}
                  placeholder="civium-invite:…"
                  rows={3}
                  className="w-full border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono
                             focus:outline-none focus:ring-2 focus:ring-civium-500 resize-none"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Votre nom dans ce réseau
                </label>
                <input
                  type="text"
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  placeholder="ex. Pierre, Alice…"
                  maxLength={MAX_TEXT}
                  className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm
                             focus:outline-none focus:ring-2 focus:ring-civium-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Adresse du pair
                  <span className="ml-1 text-xs font-normal text-gray-400">(P2P — recommandé)</span>
                </label>
                <input
                  type="text"
                  value={peerAddr}
                  onChange={(e) => setPeerAddr(e.target.value)}
                  placeholder="/ip4/1.2.3.4/tcp/4001/p2p/12D3…"
                  className="w-full border border-gray-200 rounded-lg px-3 py-2 text-xs font-mono
                             focus:outline-none focus:ring-2 focus:ring-civium-500"
                />
                <p className="text-xs text-gray-400 mt-1">
                  L'administrateur vous l'a communiquée avec le lien d'invitation (visible dans son appli, onglet Membres).
                </p>
              </div>
            </div>
            {error && (
              <div className="bg-red-50 text-red-700 text-sm rounded-lg px-4 py-3">
                {error}
              </div>
            )}
            <div className="flex gap-3">
              <button
                onClick={() => setStep("choice")}
                className="px-4 py-3 text-sm text-gray-500 hover:text-gray-700 transition-colors"
              >
                ← Retour
              </button>
              <button
                onClick={joinNetwork}
                disabled={loading || !joinInviteLink.trim() || !displayName.trim()}
                className="flex-1 py-3 bg-civium-600 text-white rounded-xl font-semibold
                           hover:bg-civium-700 disabled:opacity-50 transition-colors"
              >
                {loading ? joinLoadingLabel : "Rejoindre le réseau"}
              </button>
            </div>
          </div>
        )}

        {/* Done */}
        {step === "done" && network && (
          <div className="space-y-6">
            <div className="text-center">
              <div className="text-4xl mb-3">{mode === "create" ? "✅" : "🎉"}</div>
              <h2 className="text-xl font-bold text-gray-900">
                {mode === "create" ? "Réseau créé !" : "Vous avez rejoint le réseau !"}
              </h2>
              <p className="text-sm text-gray-500 mt-1">
                «&nbsp;{network.name}&nbsp;» — {network.member_count} membre(s)
              </p>
            </div>

            {mode === "create" && inviteLink && (
              <div>
                <p className="text-xs font-medium text-gray-600 mb-1">
                  Lien d'invitation (à transmettre) :
                </p>
                <button
                  type="button"
                  aria-label="Copier le lien d'invitation dans le presse-papiers"
                  className="w-full text-left bg-gray-50 border border-gray-200 rounded-lg p-3 text-xs
                             font-mono break-all text-gray-700 cursor-pointer
                             hover:bg-gray-100 transition-colors"
                  onClick={() => navigator.clipboard.writeText(inviteLink!)}
                >
                  {inviteLink}
                </button>
                <p className="text-xs text-gray-400 mt-1">
                  Cliquer pour copier dans le presse-papiers
                </p>
              </div>
            )}

            {identity && (
              <div className="bg-amber-50 border border-amber-200 rounded-lg p-4 text-xs space-y-1">
                <p className="font-semibold text-amber-800">
                  Sauvegardez votre clé secrète :
                </p>
                <code className="font-mono break-all text-amber-700">
                  {identity.secret_b58}
                </code>
                <p className="text-amber-600">
                  Sans cette clé, vous ne pourrez pas récupérer votre identité.
                </p>
              </div>
            )}

            <button
              onClick={onComplete}
              className="w-full py-3 bg-civium-600 text-white rounded-xl font-semibold
                         hover:bg-civium-700 transition-colors"
            >
              Accéder au tableau de bord
            </button>
          </div>
        )}
      </div>
    </main>
  );
}
