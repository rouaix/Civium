import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { IdentityInfo, NetworkInfo } from "../types";

type Step = "welcome" | "identity" | "network" | "done";

interface Props {
  onComplete: () => void;
}

export default function Onboarding({ onComplete }: Props) {
  const [step, setStep] = useState<Step>("welcome");
  const [identity, setIdentity] = useState<IdentityInfo | null>(null);
  const [network, setNetwork] = useState<NetworkInfo | null>(null);
  const [networkName, setNetworkName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [inviteLink, setInviteLink] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function createIdentity() {
    setLoading(true);
    setError(null);
    try {
      const id = await invoke<IdentityInfo>("identity_init");
      setIdentity(id);
      setStep("network");
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
      const net = await invoke<NetworkInfo>("network_create", {
        name: networkName.trim(),
        displayName: displayName.trim(),
      });
      setNetwork(net);
      const link = await invoke<string>("network_invite", {
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

        {/* Create network */}
        {step === "network" && identity && (
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
            </div>
            {error && (
              <div className="bg-red-50 text-red-700 text-sm rounded-lg px-4 py-3">
                {error}
              </div>
            )}
            <button
              onClick={createNetwork}
              disabled={loading || !networkName.trim() || !displayName.trim()}
              className="w-full py-3 bg-civium-600 text-white rounded-xl font-semibold
                         hover:bg-civium-700 disabled:opacity-50 transition-colors"
            >
              {loading ? "Création…" : "Créer le réseau"}
            </button>
          </div>
        )}

        {/* Done */}
        {step === "done" && network && (
          <div className="space-y-6">
            <div className="text-center">
              <div className="text-4xl mb-3">✅</div>
              <h2 className="text-xl font-bold text-gray-900">
                Réseau créé !
              </h2>
              <p className="text-sm text-gray-500 mt-1">
                «&nbsp;{network.name}&nbsp;» est prêt. Invitez vos premiers membres.
              </p>
            </div>
            {inviteLink && (
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
