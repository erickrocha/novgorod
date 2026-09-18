import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import enCommon from "../locales/en/common.json";
import ptBrCommon from "../locales/pt-BR/common.json";

export const resources = {
  en: {
    common: enCommon,
  },
  "pt-BR": {
    common: ptBrCommon,
  },
} as const;

export const defaultNS = "common";
export const fallbackLng = "en";

const savedLng =
  typeof window !== "undefined"
    ? localStorage.getItem("i18nextLng") || localStorage.getItem("language")
    : null;
const browserLng =
  typeof window !== "undefined" ? navigator.language : fallbackLng;
const browserBaseLng = browserLng.split("-")[0];
const initialLng =
  savedLng ||
  (resources[browserLng as keyof typeof resources]
    ? browserLng
    : resources[browserBaseLng as keyof typeof resources]
      ? browserBaseLng
      : fallbackLng);

i18n.use(initReactI18next).init({
  resources,
  lng: initialLng,
  fallbackLng,
  defaultNS,
  ns: ["common"],
  interpolation: {
    escapeValue: false, // React already escapes values
    prefix: "{",
    suffix: "}",
  },
});

export default i18n;
