import { useState, createContext, useContext, ReactNode, useEffect } from 'react';
import { useTranslation } from "react-i18next";

interface LanguageContextType {
  currentLanguage: string
  changeLanguage: (lang: string) => void;
}

const ThemeContext = createContext<LanguageContextType | undefined>(undefined);

export function LanguageProvider({children} : {children: ReactNode}) {
    const { i18n } = useTranslation(); 
    const [currentLanguage, setCurrentLanguage] = useState(""); 
    
    useEffect(() => {
        const lang = localStorage.getItem("lang"); 
            if (lang && lang.length > 0 ) { 
                i18n.changeLanguage(lang);      
                setCurrentLanguage(lang) 
            } else { 
                localStorage.setItem("lang", "en"); 
                i18n.changeLanguage("en");
                setCurrentLanguage("en") 
            }
    }, [])

    const changeLanguage = (language: string) => { 
        localStorage.setItem("lang", language); 
        i18n.changeLanguage(language);
        setCurrentLanguage(language) 
    }

    return (
        <ThemeContext.Provider value={{currentLanguage, changeLanguage}}>
            {children}
        </ThemeContext.Provider>
    )
}

export function useLanguage() {
      const context = useContext(ThemeContext);
      if (!context) throw new Error("useTheme must be used within ThemeProvider");
      return context;
 }
