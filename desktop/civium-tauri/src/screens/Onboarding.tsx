import { useState } from "react";
import { tauriInvoke } from "../tauri";
import type { IdentityInfo, NetworkInfo } from "../types";

type Step = "welcome" | "identity" | "choice" | "create" | "join" | "done";
type Mode = "create" | "join";

interface Props {
  onComplete: () => void;
}

export default function Onboarding({ onComplete }: Props) {
  const [step, setStep] = useState<Step>("welcome");
  const [mode, setMode] = useState<Mode>("create");
  const [identity, setIdentity] = useState<IdentityInfo | null>(null);
  const [network, setNetwork] = useState<NetworkInfo | null>(null);
  const [networkName, setNetworkName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [networkPrivacy, setNetworkPrivacy] = useState(false);
  const [inviteLink, setInviteLink] = useState<string | null>(null);
  const [joinInviteLink, setJoinInviteLink] = useState("");
  const [peerAddr, setPeerAddr] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function createIdentity() {
    setLoading(true);
    setError(null);
    try {
      const id = await tauriInvoke<IdentityInfo>("identity_init");
      setIdentity(id);
      setStep("choice");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function createNetwork() {
    if (!networkName.trim() || !displayName.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const net = await tauriInvoke<NetworkInfo>("network_create", {
        name: networkName.trim(),
        displayName: displayName.trim(),
        privacy: networkPrivacy,
      });
      setNetwork(net);
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
    if (!joinInviteLink.trim() || !displayName.trim()) return;
    setLoading(true);
    setError(null);
    try {
      let net: NetworkInfo;
      if (peerAddr.trim()) {
        // Phase 1: real P2P join via a live peer
        net = await tauriInvoke<NetworkInfo>("network_join_p2p", {
          inviteLink: joinInviteLink.trim(),
          displayName: displayName.trim(),
          peerAddr: peerAddr.trim(),
        });
      } else {
        // Phase 0 fallback: requires the network already in local DB
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

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-civium-50 to-civium-100 p-6">
      <div className="bg-white rounded-2xl shadow-lg max-w-md w-full p-8">

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
              Commencer
            </button>
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
              className="w-full py-3 bg-civium-600 text-white rounded-xl font-semibold
                         hover:bg-civium-700 disabled:opacity-50 transition-colors"
            >
              {loading ? "Génération…" : "Générer mon identité"}
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
                  className="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm
                             focus:outline-none focus:ring-2 focus:ring-civium-500"
                />
              </div>
              <div className="flex items-start gap-3 pt-1">
                <input
                  id="privacy-check"
                  type="checkbox"
                  checked={networkPrivacy}
                  onChange={(e) => setNetworkPrivacy(e.target.checked)}
                  className="mt-0.5 h-4 w-4 rounded border-gray-300 text-civium-600
                             focus:ring-civium-500"
                />
                <label htmlFor="privacy-check" className="text-sm text-gray-600 cursor-pointer">
                  <span className="font-medium text-gray-800">Mode privé</span>
                  <span className="block text-xs text-gray-500 mt-0.5">
                    Inscrit dans l'annuaire Civium mais non visible des autres réseaux.
                  </span>
                </label>
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
                  Obtenez-la via <code className="font-mono">civium node start --announce</code>
                  {" "}sur la machine de l'admin.
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
                <div
                  className="bg-gray-50 border border-gray-200 rounded-lg p-3 text-xs
                             font-mono break-all text-gray-700 cursor-pointer
                             hover:bg-gray-100 transition-colors"
                  onClick={() => navigator.clipboard.writeText(inviteLink!)}
                  title="Cliquer pour copier"
                >
                  {inviteLink}
                </div>
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
    </div>
  );
}
