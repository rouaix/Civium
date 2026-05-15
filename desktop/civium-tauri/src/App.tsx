import { useState, useEffect } from "react";
import { tauriInvoke, isTauriContext } from "./tauri";
import Onboarding from "./screens/Onboarding";
import Dashboard from "./screens/Dashboard";

type AppState = "loading" | "onboarding" | "dashboard";

function NoTauri() {
  return (
    <div className="flex items-center justify-center h-screen bg-gray-50">
      <div className="bg-white rounded-2xl shadow-lg max-w-sm w-full p-8 text-center space-y-4">
        <div className="text-4xl">🖥️</div>
        <h1 className="text-lg font-bold text-gray-900">Application desktop</h1>
        <p className="text-sm text-gray-500 leading-relaxed">
          Civium nécessite le runtime Tauri — il ne fonctionne pas dans un navigateur.
        </p>
        <div className="bg-gray-900 rounded-lg px-4 py-3 text-left">
          <p className="text-xs text-gray-400 mb-1">
            Depuis le dossier <code className="text-gray-300">desktop/civium-tauri</code> :
          </p>
          <code className="text-green-400 text-sm">cargo tauri dev</code>
        </div>
        <p className="text-xs text-gray-400">Une fenêtre native s'ouvrira automatiquement.</p>
      </div>
    </div>
  );
}

export default function App() {
  const [state, setState] = useState<AppState>("loading");
  const tauri = isTauriContext();

  useEffect(() => {
    if (!tauri) return;
    tauriInvoke<boolean>("identity_exists")
      .then((exists) => setState(exists ? "dashboard" : "onboarding"))
      .catch(() => setState("onboarding"));
  }, [tauri]);

  if (!tauri) return <NoTauri />;

  if (state === "loading") {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-gray-400 text-sm">Chargement…</div>
      </div>
    );
  }

  if (state === "onboarding") {
    return <Onboarding onComplete={() => setState("dashboard")} />;
  }

  return <Dashboard />;
}
