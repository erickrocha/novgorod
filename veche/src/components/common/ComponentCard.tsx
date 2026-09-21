interface ComponentCardProps {
  title?: string;
  children: React.ReactNode;
  className?: string; // Additional custom classes for styling
  desc?: string; // Description text
  action?: React.ReactNode;
}

const ComponentCard: React.FC<ComponentCardProps> = ({
  title,
  children,
  className = "",
  desc = "",
  action,
}) => {
  return (
    <div
      className={`rounded-[20px] border border-gray-200 bg-white shadow-[0_4px_14px_rgba(15,42,71,0.06)] transition-all duration-200 dark:border-gray-800 dark:bg-white/[0.03] ${className}`}
    >
      {/* Card Header */}
      {(title || desc || action) && (
        <div className="flex flex-col gap-3 px-6 py-5 sm:flex-row sm:items-center sm:justify-between">
          <div>
            {title && (
              <h3 className="font-heading text-lg font-semibold tracking-tight text-gray-900 dark:text-white/90">
                {title}
              </h3>
            )}
            {desc && (
              <p className="mt-1 text-sm text-gray-500 leading-relaxed dark:text-gray-400">
                {desc}
              </p>
            )}
          </div>
          {action && <div className="shrink-0">{action}</div>}
        </div>
      )}

      {/* Card Body */}
      <div
        className={`p-4 sm:p-6 ${
          title || desc ? "border-t border-gray-200 dark:border-gray-800" : ""
        }`}
      >
        <div className="space-y-6">{children}</div>
      </div>
    </div>
  );
};

export default ComponentCard;
