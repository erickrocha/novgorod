import {
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnDef,
  type PaginationState,
  type SortingState,
} from "@tanstack/react-table";
import { ChevronDown, ChevronUp, ChevronsUpDown } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

export interface DataGridProps<T> {
  data: T[];
  columns: ColumnDef<T, unknown>[];
  getRowId?: (row: T, index: number) => string;
  loading?: boolean;
  error?: string | null;
  emptyMessage?: string;
  pageSizeOptions?: number[];
  manualPagination?: boolean;
  manualSorting?: boolean;
  manualFiltering?: boolean;
  totalRows?: number;
  pagination?: PaginationState;
  onPaginationChange?: (pagination: PaginationState) => void;
  sorting?: SortingState;
  onSortingChange?: (sorting: SortingState) => void;
  globalFilter?: string;
  onGlobalFilterChange?: (value: string) => void;
  editable?: boolean;
  actions?: React.ReactNode;
}

export default function DataGrid<T>({
  data,
  columns,
  getRowId,
  loading = false,
  error,
  emptyMessage,
  pageSizeOptions = [10, 25, 50, 100],
  manualPagination = false,
  manualSorting = false,
  manualFiltering = false,
  totalRows,
  pagination: controlledPagination,
  onPaginationChange,
  sorting: controlledSorting,
  onSortingChange,
  globalFilter: controlledGlobalFilter,
  onGlobalFilterChange,
  actions,
}: DataGridProps<T>) {
  const { t } = useTranslation();
  const [paginationState, setPaginationState] = useState<PaginationState>(
    controlledPagination ?? { pageIndex: 0, pageSize: 10 },
  );
  const [sortingState, setSortingState] = useState<SortingState>(
    controlledSorting ?? [],
  );
  const [globalFilterState, setGlobalFilterState] = useState(
    controlledGlobalFilter ?? "",
  );

  useEffect(() => {
    if (controlledPagination) setPaginationState(controlledPagination);
  }, [controlledPagination]);
  useEffect(() => {
    if (controlledSorting) setSortingState(controlledSorting);
  }, [controlledSorting]);
  useEffect(() => {
    if (controlledGlobalFilter !== undefined)
      setGlobalFilterState(controlledGlobalFilter);
  }, [controlledGlobalFilter]);

  const table = useReactTable({
    data,
    columns,
    state: {
      pagination: paginationState,
      sorting: sortingState,
      globalFilter: globalFilterState,
    },
    onPaginationChange: (updater) => {
      const next =
        typeof updater === "function" ? updater(paginationState) : updater;
      setPaginationState(next);
      onPaginationChange?.(next);
    },
    onSortingChange: (updater) => {
      const next =
        typeof updater === "function" ? updater(sortingState) : updater;
      setSortingState(next);
      onSortingChange?.(next);
    },
    onGlobalFilterChange: (updater) => {
      const next =
        typeof updater === "function" ? updater(globalFilterState) : updater;
      setGlobalFilterState(next);
      onGlobalFilterChange?.(next);
    },
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: manualFiltering ? undefined : getFilteredRowModel(),
    getSortedRowModel: manualSorting ? undefined : getSortedRowModel(),
    getPaginationRowModel: manualPagination
      ? undefined
      : getPaginationRowModel(),
    manualPagination,
    manualSorting,
    manualFiltering,
    pageCount: manualPagination
      ? Math.max(
          1,
          Math.ceil((totalRows ?? data.length) / paginationState.pageSize),
        )
      : undefined,
    getRowId,
  });

  return (
    <div className="space-y-4">
      <div className="gap-3 flex flex-wrap items-center justify-between">
        {!manualFiltering || onGlobalFilterChange ? (
          <input
            value={globalFilterState}
            onChange={(event) => table.setGlobalFilter(event.target.value)}
            placeholder={t("common.search", "Search...")}
            className="h-9 min-w-64 rounded-lg border border-gray-200 bg-white px-3 text-sm text-gray-900 shadow-theme-xs placeholder:text-gray-400 focus:border-brand-500 focus:ring-brand-500/15 dark:border-gray-700 dark:bg-gray-900 dark:text-white/90 outline-none focus:ring-3 transition-colors"
          />
        ) : (
          <span />
        )}
        {actions && (
          <div className="flex flex-wrap items-center gap-2">
            {actions}
          </div>
        )}
      </div>

      {error && (
        <div className="rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border">
          {error}
        </div>
      )}
      <div className="rounded-xl border border-gray-200 bg-white shadow-[0_2px_8px_rgba(15,42,71,0.04)] dark:border-gray-800 dark:bg-transparent overflow-x-auto">
        <table className="text-sm min-w-full text-start">
          <thead className="bg-gray-50 border-b border-gray-200 dark:bg-white/5 dark:border-gray-800">
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  const canSort = header.column.getCanSort();
                  const sorted = header.column.getIsSorted();
                  const isActions =
                    header.id === "actions" || header.column.id === "actions";
                  return (
                    <th
                      key={header.id}
                      className={`px-4 py-2.5 text-xs font-semibold uppercase tracking-wider text-gray-500 whitespace-nowrap ${
                        isActions ? "text-end w-px" : "text-start"
                      }`}
                    >
                      {header.isPlaceholder ? null : canSort ? (
                        <button
                          type="button"
                          className="gap-1 inline-flex items-center text-start font-medium text-inherit"
                          onClick={header.column.getToggleSortingHandler()}
                        >
                          {flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                          {sorted === "asc" ? (
                            <ChevronUp size={14} />
                          ) : sorted === "desc" ? (
                            <ChevronDown size={14} />
                          ) : (
                            <ChevronsUpDown size={14} />
                          )}
                        </button>
                      ) : (
                        <span className={isActions ? "inline-block text-end" : "inline-block text-start"}>
                          {flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                        </span>
                      )}
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td
                  colSpan={columns.length}
                  className="px-4 py-6 text-gray-500 text-center text-xs"
                >
                  {t("common.loading", "Loading…")}
                </td>
              </tr>
            ) : table.getRowModel().rows.length === 0 ? (
              <tr>
                <td
                  colSpan={columns.length}
                  className="px-4 py-6 text-gray-500 text-center text-xs"
                >
                  {emptyMessage || t("common.noRecordsFound", "No records found.")}
                </td>
              </tr>
            ) : (
              table.getRowModel().rows.map((row) => (
                <tr
                  key={row.id}
                  className="border-gray-100 dark:border-gray-800 border-t hover:bg-brand-25/50 dark:hover:bg-white/[0.02] transition-colors"
                >
                  {row.getVisibleCells().map((cell) => {
                    const isActions = cell.column.id === "actions";
                    return (
                      <td
                        key={cell.id}
                        className={`px-4 py-2.5 text-sm text-gray-900 whitespace-nowrap leading-normal ${
                          isActions ? "text-end w-px" : "text-start"
                        }`}
                      >
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext(),
                        )}
                      </td>
                    );
                  })}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      <div className="gap-3 text-sm text-gray-500 flex flex-wrap items-center justify-between">
        <span>
          {manualPagination
            ? t("common.totalRecords", {
                count: totalRows ?? 0,
                defaultValue: `${totalRows ?? 0} total records`,
              })
            : t("common.records", {
                count: table.getFilteredRowModel().rows.length,
                defaultValue: `${table.getFilteredRowModel().rows.length} records`,
              })}
        </span>
        <div className="gap-2 flex items-center">
          <select
            value={paginationState.pageSize}
            onChange={(event) => table.setPageSize(Number(event.target.value))}
            className="h-9 rounded-lg border border-gray-200 bg-white px-2 text-xs font-medium text-gray-700 shadow-theme-xs focus:border-brand-500 focus:ring-brand-500/15 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300 outline-none"
          >
            {pageSizeOptions.map((size) => (
              <option key={size} value={size}>
                {t("common.perPage", { size, defaultValue: `${size} / page` })}
              </option>
            ))}
          </select>
          <button
            type="button"
            disabled={!table.getCanPreviousPage()}
            onClick={() => table.previousPage()}
            className="rounded-lg border border-gray-200 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 shadow-theme-xs hover:bg-brand-50 hover:text-brand-600 hover:border-brand-500 disabled:opacity-40 disabled:pointer-events-none transition-all duration-200 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300"
          >
            {t("common.previous", "Previous")}
          </button>
          <span className="text-xs text-gray-500">
            {t("common.pageOf", {
              current: paginationState.pageIndex + 1,
              total: Math.max(1, table.getPageCount()),
              defaultValue: `Page ${paginationState.pageIndex + 1} of ${Math.max(1, table.getPageCount())}`,
            })}
          </span>
          <button
            type="button"
            disabled={!table.getCanNextPage()}
            onClick={() => table.nextPage()}
            className="rounded-lg border border-gray-200 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 shadow-theme-xs hover:bg-brand-50 hover:text-brand-600 hover:border-brand-500 disabled:opacity-40 disabled:pointer-events-none transition-all duration-200 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300"
          >
            {t("common.next", "Next")}
          </button>
        </div>
      </div>
    </div>
  );
}
