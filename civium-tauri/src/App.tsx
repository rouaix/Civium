import { useState, useEffect } from "react";
import { tauriInvoke } from "./tauri";
import Onboarding from "./screens/Onboarding";
import Dashboard from "./screens/Dashboard";

type AppState = "loading" | "onboarding" | "dashboard";

export default function App() {
  const [state, setState] = useState<AppState>("loading");

  useEffect(() => {
    tauriInvoke<boolean>("identity_exists")
      .then((exists) => setState(exists ? "dashboard" : "onboarding"))
      .catch(() => setState("onboarding"));
  }, []);

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
