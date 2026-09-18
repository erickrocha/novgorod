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
  type VisibilityState,
} from "@tanstack/react-table";
import { ChevronDown, ChevronUp, ChevronsUpDown } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

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
}

export default function DataGrid<T>({
  data,
  columns,
  getRowId,
  loading = false,
  error,
  emptyMessage = "No records found.",
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
}: DataGridProps<T>) {
  const [paginationState, setPaginationState] = useState<PaginationState>(
    controlledPagination ?? { pageIndex: 0, pageSize: 25 },
  );
  const [sortingState, setSortingState] = useState<SortingState>(
    controlledSorting ?? [],
  );
  const [globalFilterState, setGlobalFilterState] = useState(
    controlledGlobalFilter ?? "",
  );
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});

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
      columnVisibility,
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
    onColumnVisibilityChange: setColumnVisibility,
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

  const visibleColumns = useMemo(
    () => table.getAllLeafColumns().filter((column) => column.getCanHide()),
    [table],
  );

  return (
    <div className="space-y-4">
      <div className="gap-3 flex flex-wrap items-center justify-between">
        {!manualFiltering || onGlobalFilterChange ? (
          <input
            value={globalFilterState}
            onChange={(event) => table.setGlobalFilter(event.target.value)}
            placeholder="Search..."
            className="h-11 min-w-64 rounded-lg border-gray-300 px-4 text-sm text-gray-800 focus:border-brand-300 focus:ring-brand-500/20 dark:border-gray-700 dark:text-white/90 border bg-transparent outline-none focus:ring-3"
          />
        ) : (
          <span />
        )}
        {visibleColumns.length > 0 && (
          <details className="relative">
            <summary className="rounded-lg border-gray-300 px-3 py-2 text-sm text-gray-700 dark:border-gray-700 dark:text-gray-300 cursor-pointer list-none border">
              Columns
            </summary>
            <div className="end-0 mt-2 min-w-48 rounded-lg border-gray-200 bg-white p-3 shadow-lg dark:border-gray-700 dark:bg-gray-900 absolute z-10 border">
              {visibleColumns.map((column) => (
                <label
                  key={column.id}
                  className="gap-2 py-1 text-sm flex items-center"
                >
                  <input
                    type="checkbox"
                    checked={column.getIsVisible()}
                    onChange={column.getToggleVisibilityHandler()}
                  />
                  {String(column.columnDef.header ?? column.id)}
                </label>
              ))}
            </div>
          </details>
        )}
      </div>

      {error && (
        <div className="rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border">
          {error}
        </div>
      )}
      <div className="rounded-lg border-gray-200 dark:border-gray-800 overflow-x-auto border">
        <table className="text-sm min-w-full text-start">
          <thead className="bg-gray-50 dark:bg-white/5">
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  const canSort = header.column.getCanSort();
                  const sorted = header.column.getIsSorted();
                  return (
                    <th
                      key={header.id}
                      className="px-4 py-3 font-medium text-gray-500 whitespace-nowrap"
                    >
                      {header.isPlaceholder ? null : (
                        <button
                          type="button"
                          className={
                            canSort ? "gap-1 inline-flex items-center" : ""
                          }
                          onClick={
                            canSort
                              ? header.column.getToggleSortingHandler()
                              : undefined
                          }
                        >
                          {flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                          {canSort &&
                            (sorted === "asc" ? (
                              <ChevronUp size={14} />
                            ) : sorted === "desc" ? (
                              <ChevronDown size={14} />
                            ) : (
                              <ChevronsUpDown size={14} />
                            ))}
                        </button>
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
                  className="px-4 py-12 text-gray-500 text-center"
                >
                  Loading…
                </td>
              </tr>
            ) : table.getRowModel().rows.length === 0 ? (
              <tr>
                <td
                  colSpan={columns.length}
                  className="px-4 py-12 text-gray-500 text-center"
                >
                  {emptyMessage}
                </td>
              </tr>
            ) : (
              table.getRowModel().rows.map((row) => (
                <tr
                  key={row.id}
                  className="border-gray-100 dark:border-gray-800 border-t"
                >
                  {row.getVisibleCells().map((cell) => (
                    <td key={cell.id} className="px-4 py-4">
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </td>
                  ))}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      <div className="gap-3 text-sm text-gray-500 flex flex-wrap items-center justify-between">
        <span>
          {manualPagination
            ? `${totalRows ?? 0} total records`
            : `${table.getFilteredRowModel().rows.length} records`}
        </span>
        <div className="gap-2 flex items-center">
          <select
            value={paginationState.pageSize}
            onChange={(event) => table.setPageSize(Number(event.target.value))}
            className="h-9 rounded-lg border-gray-300 px-2 dark:border-gray-700 border bg-transparent"
          >
            {pageSizeOptions.map((size) => (
              <option key={size} value={size}>
                {size} / page
              </option>
            ))}
          </select>
          <button
            type="button"
            disabled={!table.getCanPreviousPage()}
            onClick={() => table.previousPage()}
            className="rounded px-3 py-1.5 border disabled:opacity-40"
          >
            Previous
          </button>
          <span>
            Page {paginationState.pageIndex + 1} of{" "}
            {Math.max(1, table.getPageCount())}
          </span>
          <button
            type="button"
            disabled={!table.getCanNextPage()}
            onClick={() => table.nextPage()}
            className="rounded px-3 py-1.5 border disabled:opacity-40"
          >
            Next
          </button>
        </div>
      </div>
    </div>
  );
}
