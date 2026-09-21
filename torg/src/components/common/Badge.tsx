import React from 'react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: 'amber' | 'emerald' | 'rose' | 'indigo' | 'slate';
  size?: 'sm' | 'md';
}

export const Badge: React.FC<BadgeProps> = ({
  children,
  variant = 'amber',
  size = 'md',
  className,
  ...props
}) => {
  const base = 'inline-flex items-center font-semibold rounded-full tracking-wide uppercase';

  const sizes = {
    sm: 'px-2 py-0.5 text-[10px]',
    md: 'px-2.5 py-1 text-xs',
  }[size];

  const variants = {
    amber: 'bg-amber-100 text-amber-800 border border-amber-200/60',
    emerald: 'bg-emerald-100 text-emerald-800 border border-emerald-200/60',
    rose: 'bg-rose-100 text-rose-800 border border-rose-200/60',
    indigo: 'bg-indigo-100 text-indigo-800 border border-indigo-200/60',
    slate: 'bg-slate-100 text-slate-700 border border-slate-200',
  }[variant];

  return (
    <span className={twMerge(clsx(base, sizes, variants, className))} {...props}>
      {children}
    </span>
  );
};

export default Badge;
