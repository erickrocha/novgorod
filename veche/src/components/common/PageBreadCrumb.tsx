import { Link } from "react-router-dom";

export interface BreadcrumbItem {
  label: string;
  href?: string;
}

interface BreadcrumbProps {
  pageTitle: string;
  showTitle?: boolean;
  items?: BreadcrumbItem[];
}

const ChevronIcon = () => (
  <svg
    className="stroke-current rtl:rotate-180 shrink-0"
    width="17"
    height="16"
    viewBox="0 0 17 16"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      d="M6.0765 12.667L10.2432 8.50033L6.0765 4.33366"
      stroke=""
      strokeWidth="1.2"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  </svg>
);

const PageBreadcrumb: React.FC<BreadcrumbProps> = ({
  pageTitle,
  showTitle = false,
  items = [],
}) => {
  const breadcrumbNav = (
    <nav>
      <ol className="flex flex-wrap items-center gap-1.5">
        <li>
          <Link
            className="inline-flex items-center gap-1.5 text-sm text-gray-500 hover:text-brand-500 dark:text-gray-400 dark:hover:text-brand-400 transition-colors"
            to="/"
          >
            Home
            <ChevronIcon />
          </Link>
        </li>
        {items.map((item, idx) => (
          <li key={idx} className="inline-flex items-center gap-1.5">
            {item.href ? (
              <Link
                className="inline-flex items-center gap-1.5 text-sm text-gray-500 hover:text-brand-500 dark:text-gray-400 dark:hover:text-brand-400 transition-colors"
                to={item.href}
              >
                {item.label}
                <ChevronIcon />
              </Link>
            ) : (
              <span className="inline-flex items-center gap-1.5 text-sm text-gray-500 dark:text-gray-400">
                {item.label}
                <ChevronIcon />
              </span>
            )}
          </li>
        ))}
        <li className="text-sm font-medium text-gray-900 dark:text-white/90">
          {pageTitle}
        </li>
      </ol>
    </nav>
  );

  if (showTitle) {
    return (
      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-white/90">
          {pageTitle}
        </h2>
        {breadcrumbNav}
      </div>
    );
  }

  return <div className="mb-4 flex items-center justify-start">{breadcrumbNav}</div>;
};

export default PageBreadcrumb;

