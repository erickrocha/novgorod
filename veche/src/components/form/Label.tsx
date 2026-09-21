import { cn } from "@/utils";
import type { FC, ReactNode } from "react";

interface LabelProps {
  htmlFor?: string;
  children: ReactNode;
  className?: string;
}

const Label: FC<LabelProps> = ({ htmlFor, children, className }) => {
  return (
    <label
      htmlFor={htmlFor}
      className={cn(
        "mb-1.5 block text-[0.82rem] font-medium text-gray-900 dark:text-gray-300",
        className,
      )}
    >
      {children}
    </label>
  );
};

export default Label;
