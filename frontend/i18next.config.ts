import { defineConfig } from 'i18next-cli';

export default defineConfig({
  locales: [
    "en", "pl"
  ],
  extract: {
    input: "src/**/*.{js,jsx,ts,tsx}",
    output: "locales/{{language}}/{{namespace}}.json"
  }
});