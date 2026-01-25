import { createContext, useContext, useState, ReactNode } from "react";

interface UIPreferencesContextType {
  transfersEnabled: boolean;
  setTransfersEnabled: (enabled: boolean) => void;
}

const UIPreferencesContext = createContext<UIPreferencesContextType | undefined>(undefined);

const STORAGE_KEY = "uiPreferences";

export function UIPreferencesProvider({ children }: { children: ReactNode }) {
  const [transfersEnabled, setTransfersEnabledState] = useState<boolean>(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        const prefs = JSON.parse(stored);
        return prefs.transfersEnabled ?? true;
      } catch {
        return true;
      }
    }
    return true; // Default: transfers enabled
  });

  const setTransfersEnabled = (enabled: boolean) => {
    setTransfersEnabledState(enabled);
    const stored = localStorage.getItem(STORAGE_KEY);
    const prefs = stored ? JSON.parse(stored) : {};
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ ...prefs, transfersEnabled: enabled }));
  };

  return (
    <UIPreferencesContext.Provider value={{ transfersEnabled, setTransfersEnabled }}>
      {children}
    </UIPreferencesContext.Provider>
  );
}

export function useUIPreferences() {
  const context = useContext(UIPreferencesContext);
  if (context === undefined) {
    throw new Error("useUIPreferences must be used within UIPreferencesProvider");
  }
  return context;
}
