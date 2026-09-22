import { createSlice, type PayloadAction } from "@reduxjs/toolkit";
import i18n from "@/i18n";

export type Theme = "light" | "dark";
export type LanguageCode = "pt-BR" | "en";

export interface Language {
  code: LanguageCode;
  name: string;
  dir: "ltr" | "rtl";
  flag?: string;
}

export const AVAILABLE_LANGUAGES: Language[] = [
  { code: "pt-BR", name: "Português (Brasil)", dir: "ltr" },
  { code: "en", name: "English", dir: "ltr" },
];

export interface UiState {
  theme: Theme;
  language: LanguageCode;
  isExpanded: boolean;
  isMobileOpen: boolean;
  isMobile: boolean;
  isHovered: boolean;
  activeItem: string | null;
  openSubmenu: string | null;
}

const getInitialTheme = (): Theme => {
  if (typeof window !== "undefined") {
    const saved = localStorage.getItem("theme");
    if (saved === "dark" || saved === "light") return saved;
  }
  return "light";
};

const getInitialLanguage = (): LanguageCode => {
  if (typeof window !== "undefined") {
    const saved =
      localStorage.getItem("language") || localStorage.getItem("i18nextLng");
    if (saved === "en" || saved === "pt-BR") return saved;
  }
  return "pt-BR";
};

const initialState: UiState = {
  theme: getInitialTheme(),
  language: getInitialLanguage(),
  isExpanded: true,
  isMobileOpen: false,
  isMobile: false,
  isHovered: false,
  activeItem: null,
  openSubmenu: null,
};

export const uiSlice = createSlice({
  name: "ui",
  initialState,
  reducers: {
    setTheme(state, action: PayloadAction<Theme>) {
      state.theme = action.payload;
      if (typeof window !== "undefined") {
        localStorage.setItem("theme", action.payload);
        if (action.payload === "dark") {
          document.documentElement.classList.add("dark");
          document.documentElement.setAttribute("data-color-scheme", "dark");
        } else {
          document.documentElement.classList.remove("dark");
          document.documentElement.setAttribute("data-color-scheme", "light");
        }
      }
    },
    toggleTheme(state) {
      const nextTheme = state.theme === "light" ? "dark" : "light";
      state.theme = nextTheme;
      if (typeof window !== "undefined") {
        localStorage.setItem("theme", nextTheme);
        if (nextTheme === "dark") {
          document.documentElement.classList.add("dark");
          document.documentElement.setAttribute("data-color-scheme", "dark");
        } else {
          document.documentElement.classList.remove("dark");
          document.documentElement.setAttribute("data-color-scheme", "light");
        }
      }
    },
    setLanguage(state, action: PayloadAction<LanguageCode>) {
      state.language = action.payload;
      const langObj =
        AVAILABLE_LANGUAGES.find((l) => l.code === action.payload) ||
        AVAILABLE_LANGUAGES[0];
      if (typeof window !== "undefined") {
        document.documentElement.lang = action.payload;
        document.documentElement.dir = langObj.dir;
        localStorage.setItem("language", action.payload);
        localStorage.setItem("i18nextLng", action.payload);
      }
      i18n.changeLanguage(action.payload);
    },
    toggleSidebar(state) {
      state.isExpanded = !state.isExpanded;
    },
    toggleMobileSidebar(state) {
      state.isMobileOpen = !state.isMobileOpen;
    },
    setIsMobileOpen(state, action: PayloadAction<boolean>) {
      state.isMobileOpen = action.payload;
    },
    setIsMobile(state, action: PayloadAction<boolean>) {
      state.isMobile = action.payload;
      if (!action.payload) {
        state.isMobileOpen = false;
      }
    },
    setIsHovered(state, action: PayloadAction<boolean>) {
      state.isHovered = action.payload;
    },
    setActiveItem(state, action: PayloadAction<string | null>) {
      state.activeItem = action.payload;
    },
    setOpenSubmenu(state, action: PayloadAction<string | null>) {
      state.openSubmenu = action.payload;
    },
    toggleSubmenu(state, action: PayloadAction<string>) {
      state.openSubmenu =
        state.openSubmenu === action.payload ? null : action.payload;
    },
  },
});

export const {
  setTheme,
  toggleTheme,
  setLanguage,
  toggleSidebar,
  toggleMobileSidebar,
  setIsMobileOpen,
  setIsMobile,
  setIsHovered,
  setActiveItem,
  setOpenSubmenu,
  toggleSubmenu,
} = uiSlice.actions;

export default uiSlice.reducer;
