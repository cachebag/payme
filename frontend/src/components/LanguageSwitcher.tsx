import { Select } from "./ui/Select";
interface SupportedLanguageOptions { 
    label: string, 
    value: string, 
}
import { useLanguage } from "../context/LanguageContext";
export function LanguageSwitcher() 
{
    const {currentLanguage, changeLanguage} = useLanguage();

    const supported_languages: SupportedLanguageOptions[] = [
        {label: "English", value: "en"},
        {label: "Polski", value: "pl"}
    ];

    return (
        <div className="backdrop-blur-md">
            <Select className="!bg-sand-50/80! dark:bg-charcoal-950/80!" defaultValue={currentLanguage} options={supported_languages} onChange={(e) => changeLanguage(e.target.value)}></Select>
        </div>
    )
}