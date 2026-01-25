import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App";
import { ThemeProvider } from "./context/ThemeContext";
import { AuthProvider } from "./context/AuthContext";
import { CurrencyProvider } from "./context/CurrencyContext";
import { UIPreferencesProvider } from "./context/UIPreferencesContext";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider>
      <CurrencyProvider>
        <UIPreferencesProvider>
          <AuthProvider>
            <App />
          </AuthProvider>
        </UIPreferencesProvider>
      </CurrencyProvider>
    </ThemeProvider>
  </StrictMode>
);

