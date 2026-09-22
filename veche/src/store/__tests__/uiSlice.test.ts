import { describe, it, expect } from "vitest";
import uiReducer, {
  setLanguage,
  setIsHovered,
  setIsMobile,
  setIsMobileOpen,
  setTheme,
  toggleSidebar,
  toggleTheme,
} from "../uiSlice";

describe("uiSlice reducer", () => {
  const initialState = {
    theme: "light" as const,
    language: "pt-BR" as const,
    isExpanded: true,
    isMobileOpen: false,
    isMobile: false,
    isHovered: false,
    activeItem: null,
    openSubmenu: null,
  };

  it("should return the initial state when passed an empty action", () => {
    const result = uiReducer(undefined, { type: "" });
    expect(result.theme).toBeDefined();
    expect(result.language).toBeDefined();
  });

  it("should handle toggleTheme", () => {
    const nextState = uiReducer(initialState, toggleTheme());
    expect(nextState.theme).toBe("dark");

    const backToLight = uiReducer(nextState, toggleTheme());
    expect(backToLight.theme).toBe("light");
  });

  it("should handle setTheme", () => {
    const nextState = uiReducer(initialState, setTheme("dark"));
    expect(nextState.theme).toBe("dark");
  });

  it("should handle setLanguage", () => {
    const nextState = uiReducer(initialState, setLanguage("en"));
    expect(nextState.language).toBe("en");
  });

  it("should handle toggleSidebar", () => {
    const nextState = uiReducer(initialState, toggleSidebar());
    expect(nextState.isExpanded).toBe(false);

    const backExpanded = uiReducer(nextState, toggleSidebar());
    expect(backExpanded.isExpanded).toBe(true);
  });

  it("should handle setIsHovered and setIsMobile", () => {
    let state = uiReducer(initialState, setIsHovered(true));
    expect(state.isHovered).toBe(true);

    state = uiReducer(state, setIsMobile(true));
    expect(state.isMobile).toBe(true);

    state = uiReducer(state, setIsMobileOpen(true));
    expect(state.isMobileOpen).toBe(true);
  });
});
