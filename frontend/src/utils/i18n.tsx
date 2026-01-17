import i18n from 'i18next';
import { initReactI18next } from 'react-i18next'; 
import enTranslation from "../../locales/en/translation.json";
import plTranslation from "../../locales/pl/translation.json";

i18n
    .use(initReactI18next)
    .init({
        lng: 'en', 
        fallbackLng: 'en', 
        supportedLngs: ['en', 'pl'], 
        ns: ['translation'], 
        defaultNS: 'translation', 
        interpolation: { escapeValue: false}, 
        resources: { 
            en: { translation: enTranslation },
            pl: { translation: plTranslation}
        }
    })

export default i18n