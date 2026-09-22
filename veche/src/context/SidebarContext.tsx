import type React from "react";
import { useEffect } from "react";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
  setActiveItem,
  setIsHovered,
  setIsMobile,
  setIsMobileOpen,
  toggleMobileSidebar,
  toggleSidebar,
  toggleSubmenu,
} from "@/store/uiSlice";

export const SidebarProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const dispatch = useAppDispatch();

  useEffect(() => {
    const handleResize = () => {
      const mobile = window.innerWidth < 1280;
      dispatch(setIsMobile(mobile));
    };

    handleResize();
    window.addEventListener("resize", handleResize);
    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, [dispatch]);

  return <>{children}</>;
};

export const useSidebar = () => {
  const dispatch = useAppDispatch();
  const {
    isExpanded,
    isMobileOpen,
    isMobile,
    isHovered,
    activeItem,
    openSubmenu,
  } = useAppSelector((state) => state.ui);

  return {
    isExpanded: isMobile ? false : isExpanded,
    isMobileOpen,
    isHovered,
    activeItem,
    openSubmenu,
    toggleSidebar: () => dispatch(toggleSidebar()),
    toggleMobileSidebar: () => dispatch(toggleMobileSidebar()),
    setIsHovered: (hovered: boolean) => dispatch(setIsHovered(hovered)),
    setActiveItem: (item: string | null) => dispatch(setActiveItem(item)),
    toggleSubmenu: (item: string) => dispatch(toggleSubmenu(item)),
    setIsMobileOpen: (open: boolean) => dispatch(setIsMobileOpen(open)),
  };
};

