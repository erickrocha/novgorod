import type React from "react";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
  AVAILABLE_LANGUAGES,
  setLanguage,
  type Language,
  type LanguageCode,
} from "@/store/uiSlice";

export { AVAILABLE_LANGUAGES, type Language, type LanguageCode };

export const LanguageProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  return <>{children}</>;
};

export const useLanguage = () => {
  const dispatch = useAppDispatch();
  const language = useAppSelector((state) => state.ui.language);

  const currentLanguage =
    AVAILABLE_LANGUAGES.find((lang) => lang.code === language) ||
    AVAILABLE_LANGUAGES[0];
  const dir = currentLanguage.dir;

  return {
    language,
    currentLanguage,
    dir,
    setLanguage: (code: LanguageCode) => dispatch(setLanguage(code)),
    availableLanguages: AVAILABLE_LANGUAGES,
  };
};

