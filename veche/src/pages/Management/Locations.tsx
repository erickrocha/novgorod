import { useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Pencil, Plus, RefreshCw, Upload, X } from "lucide-react";
import Papa from "papaparse";
import type { ColumnDef, PaginationState, SortingState } from "@tanstack/react-table";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Button from "@/components/ui/button/Button";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
  clearLocationError,
  fetchCities,
  fetchProvinces,
  importCities,
  importProvinces,
  saveCity,
  saveProvince,
} from "@/store/locationSlice";
import { locationService } from "@/services/locationService";
import type { City, Province } from "@/services/types";
import DataGrid from "@/components/data-grid/DataGrid";

interface ProvinceCsvRow {
  ibge_code: string;
  acronym: string;
  name: string;
  country_code: string;
}

const provinceColumns = [
  "ibge_code",
  "acronym",
  "name",
  "country_code",
] as const;
const isProvinceRowInvalid = (row: ProvinceCsvRow) =>
  !row.ibge_code.trim() ||
  !row.acronym.trim() ||
  !row.name.trim() ||
  row.country_code.trim().length !== 2;

export default function Locations({ kind }: { kind: "provinces" | "cities" }) {
  const [searchParams, setSearchParams] = useSearchParams();
  const dispatch = useAppDispatch();
  const state = useAppSelector((s) => s.location);
  const [allProvinces, setAllProvinces] = useState<Province[]>([]);
  const [editing, setEditing] = useState<Province | City | null>(null);
  const [importRows, setImportRows] = useState<ProvinceCsvRow[] | null>(null);
  const [importName, setImportName] = useState("");
  const [importError, setImportError] = useState("");

  const page = Number(searchParams.get("page") || "1");
  const pageSize = Number(searchParams.get("pageSize") || "10");
  const q = searchParams.get("q") || "";
  const sortBy = searchParams.get("sortBy") || "id";
  const sortDir = (searchParams.get("sortDir") as "asc" | "desc") || "asc";

  useEffect(() => {
    dispatch(clearLocationError());
    if (kind === "provinces") {
      dispatch(fetchProvinces({ page, pageSize, q, sortBy, sortDir }));
    } else {
      dispatch(fetchCities({ page, pageSize, q, sortBy, sortDir }));
      locationService.provinces().then(setAllProvinces).catch(() => {});
    }
    return () => {
      dispatch(clearLocationError());
    };
  }, [dispatch, kind, page, pageSize, q, sortBy, sortDir]);

  useEffect(() => {
    if (!state.report && !state.error && !importError) return;
    const timeout = window.setTimeout(() => {
      dispatch(clearLocationError());
      setImportError("");
    }, 2000);
    return () => window.clearTimeout(timeout);
  }, [dispatch, importError, state.error, state.report]);

  const onPaginationChange = (next: PaginationState) => {
    const params = new URLSearchParams(searchParams);
    params.set("page", String(next.pageIndex + 1));
    params.set("pageSize", String(next.pageSize));
    setSearchParams(params);
  };

  const onSortingChange = (next: SortingState) => {
    const params = new URLSearchParams(searchParams);
    if (next.length > 0) {
      params.set("sortBy", next[0].id);
      params.set("sortDir", next[0].desc ? "desc" : "asc");
    } else {
      params.delete("sortBy");
      params.delete("sortDir");
    }
    params.set("page", "1");
    setSearchParams(params);
  };

  const onGlobalFilterChange = (val: string) => {
    const params = new URLSearchParams(searchParams);
    if (val) {
      params.set("q", val);
    } else {
      params.delete("q");
    }
    params.set("page", "1");
    setSearchParams(params);
  };

  const rows = kind === "provinces" ? state.provinces : state.cities;
  const total = kind === "provinces" ? state.provincesTotal : state.citiesTotal;
  const invalidRows = importRows?.filter(isProvinceRowInvalid).length ?? 0;

  const gridColumns: ColumnDef<Province | City, unknown>[] = useMemo(() => {
    const columns: ColumnDef<Province | City, unknown>[] = [
      {
        header: "IBGE code",
        accessorKey: "ibgeCode",
        enableSorting: true,
        cell: ({ getValue }) => getValue<string>() || "—",
      },
      {
        header: "Name",
        accessorKey: "name",
        enableSorting: true,
        cell: ({ getValue }) => (
          <span className="font-medium">{getValue<string>()}</span>
        ),
      },
    ];
    if (kind === "provinces") {
      columns.push({ header: "UF", accessorKey: "acronym", enableSorting: true });
    }
    columns.push({
      id: "actions",
      header: "",
      enableSorting: false,
      enableHiding: false,
      cell: ({ row }) => (
        <div className="flex items-center justify-end text-end">
          <button
            type="button"
            title="Edit"
            aria-label="Edit"
            onClick={() => setEditing(row.original)}
            className="h-7 w-7 rounded-md inline-flex items-center justify-center text-gray-500 hover:text-brand-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:text-brand-400 dark:hover:bg-white/5 transition-colors"
          >
            <Pencil size={15} />
          </button>
        </div>
      ),
    });
    return columns;
  }, [kind]);

  const submit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    if (kind === "provinces") {
      dispatch(
        saveProvince({
          ...((editing as Province) || {}),
          ibgeCode: String(form.get("ibgeCode")),
          acronym: String(form.get("acronym")),
          name: String(form.get("name")),
          countryCode: "BR",
        }),
      );
    } else {
      dispatch(
        saveCity({
          ...((editing as City) || {}),
          ibgeCode: String(form.get("ibgeCode")),
          provinceId: Number(form.get("provinceId")),
          name: String(form.get("name")),
        }),
      );
    }
    setEditing(null);
  };

  const chooseImport = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    if (kind === "cities") {
      dispatch(importCities(file));
      return;
    }
    dispatch(clearLocationError());
    setImportRows(null);
    setImportName(file.name);
    setImportError("");
    Papa.parse<ProvinceCsvRow>(file, {
      header: true,
      skipEmptyLines: true,
      transformHeader: (header) => header.trim().toLowerCase(),
      complete: (result) => {
        const missing = provinceColumns.filter(
          (header) => !result.meta.fields?.includes(header),
        );
        if (missing.length) {
          setImportRows(null);
          setImportError(`Missing CSV columns: ${missing.join(", ")}.`);
        } else if (result.errors.length) {
          setImportRows(null);
          setImportError(result.errors[0].message);
        } else {
          setImportName(file.name);
          setImportRows(
            result.data.map((row) => ({
              ibge_code: String(row.ibge_code ?? "").trim(),
              acronym: String(row.acronym ?? "")
                .trim()
                .toUpperCase(),
              name: String(row.name ?? "").trim(),
              country_code: String(row.country_code ?? "")
                .trim()
                .toUpperCase(),
            })),
          );
        }
      },
      error: (error) => {
        setImportRows(null);
        setImportError(error.message);
      },
    });
  };

  const updateImportRow = (
    index: number,
    field: keyof ProvinceCsvRow,
    value: string,
  ) => {
    setImportRows(
      (current) =>
        current?.map((row, rowIndex) =>
          rowIndex === index ? { ...row, [field]: value } : row,
        ) ?? null,
    );
  };
  const closePreview = () => {
    setImportRows(null);
    setImportName("");
    setImportError("");
  };
  const saveImport = async () => {
    if (!importRows?.length || invalidRows) return;
    const csv = Papa.unparse(importRows, {
      columns: [...provinceColumns],
      newline: "\n",
    });
    const file = new File([csv], importName || "provinces.csv", {
      type: "text/csv;charset=utf-8",
    });
    try {
      await dispatch(importProvinces(file)).unwrap();
      closePreview();
      dispatch(fetchProvinces({ page, pageSize, q, sortBy, sortDir }));
    } catch {
      // Keep the edited preview open; the Redux request error is shown below.
    }
  };

  return (
    <>
      <PageMeta
        title={`${kind === "provinces" ? "Provinces" : "Cities"} | Veche`}
        description="Manage Brazilian locations"
      />
      <PageBreadcrumb
        pageTitle={kind === "provinces" ? "Provinces" : "Cities"}
        showTitle={false}
      />
      <ComponentCard>
        {kind === "provinces" && importError && (
          <div className="mb-4 rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border">
            {importError}
          </div>
        )}
        {kind === "provinces" && importRows && (
          <div className="mb-6 rounded-lg border-gray-200 p-4 dark:border-gray-800 border">
            <div className="mb-4 gap-3 flex flex-wrap items-start justify-between">
              <div>
                <h3 className="font-semibold text-gray-800 dark:text-white/90">
                  Review province import
                </h3>
                <p className="text-sm text-gray-500">
                  {importName} · {importRows.length} rows. Edit any value before
                  saving.
                </p>
              </div>
              <button
                type="button"
                aria-label="Close import preview"
                className="text-gray-500 hover:text-gray-800 dark:hover:text-white"
                onClick={closePreview}
              >
                <X size={20} />
              </button>
            </div>
            {invalidRows > 0 && (
              <div className="mb-4 rounded-lg border-warning-200 bg-warning-50 p-3 text-sm text-warning-700 border">
                {invalidRows} row{invalidRows === 1 ? " has" : "s have"} missing
                or invalid required values. Fix them before saving.
              </div>
            )}
            {importRows.length === 0 && (
              <div className="mb-4 rounded-lg border-warning-200 bg-warning-50 p-3 text-sm text-warning-700 border">
                The selected CSV has no province rows.
              </div>
            )}
            <div className="max-h-[55vh] overflow-auto">
              <table className="text-sm w-full min-w-[760px] text-start">
                <thead>
                  <tr className="border-gray-100 dark:border-gray-800 border-b">
                    <th className="px-2 py-3">#</th>
                    <th className="px-2 py-3">IBGE code</th>
                    <th className="px-2 py-3">UF</th>
                    <th className="px-2 py-3">Name</th>
                    <th className="px-2 py-3">Country</th>
                  </tr>
                </thead>
                <tbody>
                  {importRows.map((row, index) => (
                    <tr
                      key={index}
                      className="border-gray-100 dark:border-gray-800 border-b"
                    >
                      <td className="px-2 py-2 text-gray-500">{index + 1}</td>
                      {provinceColumns.map((field) => (
                        <td key={field} className="px-2 py-2">
                          <Input
                            value={row[field]}
                            error={
                              !row[field].trim() ||
                              (field === "country_code" &&
                                row[field].trim().length !== 2)
                            }
                            aria-label={`Row ${index + 1} ${field.replace("_", " ")}`}
                            onChange={(event) =>
                              updateImportRow(index, field, event.target.value)
                            }
                          />
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div className="mt-5 gap-3 flex justify-end">
              <Button type="button" variant="outline" onClick={closePreview}>
                Cancel
              </Button>
              <Button
                type="button"
                disabled={
                  !importRows.length || invalidRows > 0 || state.loading
                }
                onClick={saveImport}
              >
                {state.loading ? "Saving…" : "Save"}
              </Button>
            </div>
          </div>
        )}

        {editing && (
          <form
            onSubmit={submit}
            className="mb-6 gap-4 rounded-lg border-gray-200 p-4 md:grid-cols-4 grid border"
          >
            <div>
              <Label htmlFor="ibgeCode">IBGE code</Label>
              <Input
                id="ibgeCode"
                name="ibgeCode"
                defaultValue={editing.ibgeCode || ""}
                required
              />
            </div>
            {kind === "provinces" && (
              <div>
                <Label htmlFor="acronym">Acronym</Label>
                <Input
                  id="acronym"
                  name="acronym"
                  defaultValue={(editing as Province).acronym}
                  required
                />
              </div>
            )}
            {kind === "cities" && (
              <div>
                <Label htmlFor="provinceId">Province</Label>
                <select
                  id="provinceId"
                  name="provinceId"
                  defaultValue={(editing as City).provinceId}
                  className="h-11 rounded-lg border-gray-300 px-3 text-sm w-full border"
                >
                  {allProvinces.map((province) => (
                    <option key={province.id} value={province.id}>
                      {province.name}
                    </option>
                  ))}
                </select>
              </div>
            )}
            <div>
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                name="name"
                defaultValue={editing.name}
                required
              />
            </div>
            <div className="gap-2 flex items-end">
              <Button type="submit">Save</Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => setEditing(null)}
              >
                Cancel
              </Button>
            </div>
          </form>
        )}
        {state.error && (
          <div className="mb-4 rounded-lg border-error-200 bg-error-50 p-3 text-sm text-error-600 border">
            {state.error}
          </div>
        )}
        {state.report && (
          <div className="mb-4 rounded-lg border-success-200 bg-success-50 p-3 text-sm border">
            Imported: {state.report.inserted} inserted, {state.report.updated}{" "}
            updated, {state.report.skipped} skipped.
          </div>
        )}
        <DataGrid
          data={rows}
          columns={gridColumns}
          actions={
            <div className="gap-2 flex flex-wrap items-center">
              <Button
                size="sm"
                variant="outline"
                startIcon={<RefreshCw size={14} />}
                onClick={() => {
                  if (kind === "provinces") {
                    dispatch(fetchProvinces({ page, pageSize, q, sortBy, sortDir }));
                  } else {
                    dispatch(fetchCities({ page, pageSize, q, sortBy, sortDir }));
                  }
                }}
              >
                Refresh
              </Button>
              <label className="rounded-lg bg-brand-500 px-3 py-1.5 text-xs font-medium text-white inline-flex cursor-pointer items-center shadow-theme-xs hover:bg-brand-600 transition-colors">
                <Upload size={14} className="me-1.5" />
                Import CSV
                <input
                  type="file"
                  accept=".csv,text/csv"
                  className="hidden"
                  onChange={chooseImport}
                />
              </label>
              <Button
                size="sm"
                startIcon={<Plus size={14} />}
                onClick={() =>
                  setEditing(
                    kind === "provinces"
                      ? { acronym: "", name: "", countryCode: "BR", ibgeCode: "" }
                      : {
                          provinceId: allProvinces[0]?.id || 0,
                          name: "",
                          ibgeCode: "",
                        },
                  )
                }
              >
                Add
              </Button>
            </div>
          }
          getRowId={(row, index) => String(row.id ?? row.ibgeCode ?? index)}
          loading={state.loading && rows.length === 0}
          error={state.error}
          emptyMessage={`No ${kind} found.`}
          manualPagination
          manualSorting
          manualFiltering
          totalRows={total}
          pagination={{ pageIndex: Math.max(0, page - 1), pageSize }}
          onPaginationChange={onPaginationChange}
          sorting={[{ id: sortBy, desc: sortDir === "desc" }]}
          onSortingChange={onSortingChange}
          globalFilter={q}
          onGlobalFilterChange={onGlobalFilterChange}
        />
      </ComponentCard>
    </>
  );
}
