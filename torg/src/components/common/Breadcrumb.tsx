import React from 'react';
import { Link } from 'react-router-dom';
import { ChevronRight, Home } from 'lucide-react';

export interface BreadcrumbItem {
  label: string;
  href?: string;
}

interface BreadcrumbProps {
  items: BreadcrumbItem[];
  className?: string;
  showHome?: boolean;
}

export const Breadcrumb: React.FC<BreadcrumbProps> = ({
  items,
  className = '',
  showHome = true,
}) => {
  return (
    <nav aria-label="Breadcrumb" className={`flex items-center text-xs font-medium text-slate-500 ${className}`}>
      <ol className="inline-flex items-center space-x-1.5 md:space-x-2">
        {showHome && (
          <li className="inline-flex items-center">
            <Link
              to="/"
              className="inline-flex items-center gap-1 text-slate-500 hover:text-amber-600 transition-colors"
            >
              <Home className="w-3.5 h-3.5" />
              <span className="sr-only sm:not-sr-only">Início</span>
            </Link>
          </li>
        )}


        {items.map((item, index) => {
          const isLast = index === items.length - 1;
          const showSeparator = showHome || index > 0;

          return (
            <li key={index} className="inline-flex items-center">
              {showSeparator && (
                <ChevronRight className="w-3.5 h-3.5 text-slate-400 mx-1 shrink-0" />
              )}
              {item.href && !isLast ? (

                <Link
                  to={item.href}
                  className="text-slate-500 hover:text-amber-600 transition-colors"
                >
                  {item.label}
                </Link>
              ) : (
                <span className="text-slate-900 font-semibold" aria-current={isLast ? 'page' : undefined}>
                  {item.label}
                </span>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
};

export default Breadcrumb;
