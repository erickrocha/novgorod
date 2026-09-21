import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { PencilIcon, PlusIcon, TrashBinIcon } from "../../icons";
import Button from "../ui/button/Button";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
  deletePersonAddress,
  fetchPersonAddresses,
} from "@/store/authSlice";
import { maskCep } from "@/utils/mask";

export default function UserAddressCard() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { person, addresses } = useAppSelector((state) => state.auth);
  const [deletingId, setDeletingId] = useState<number | null>(null);

  useEffect(() => {
    if (person?.id && addresses.length === 0) {
      dispatch(fetchPersonAddresses(person.id));
    }
  }, [dispatch, person?.id, addresses.length]);

  const handleDelete = async (id: number) => {
    if (!window.confirm(t("profile.address.confirmDelete", "Tem certeza que deseja excluir este endereço?"))) {
      return;
    }

    try {
      setDeletingId(id);
      await dispatch(deletePersonAddress(id)).unwrap();
    } catch (err: unknown) {
      console.error("Failed to delete address", err);
    } finally {
      setDeletingId(null);
    }
  };

  return (
    <div className="rounded-2xl border border-gray-200 bg-white p-5 shadow-theme-xs lg:p-6 dark:border-gray-800 dark:bg-white/[0.03]">
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <h4 className="text-lg font-semibold text-gray-900 dark:text-white/90">
          {t("profile.address.title")}
        </h4>
        <Link to="/profile/addresses/new">
          <Button
            size="sm"
            className="inline-flex items-center gap-2"
          >
            <PlusIcon className="size-4" />
            {t("profile.address.addAddress")}
          </Button>
        </Link>
      </div>

      {addresses.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-gray-300 py-10 text-center dark:border-gray-700">
          <p className="mb-4 text-sm text-gray-500 dark:text-gray-400">
            {t("profile.address.noAddresses")}
          </p>
          <Link to="/profile/addresses/new">
            <Button
              size="sm"
              variant="outline"
              className="inline-flex items-center gap-2"
            >
              <PlusIcon className="size-4" />
              {t("profile.address.addAddress")}
            </Button>
          </Link>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {addresses.map((addr, idx) => (
            <div
              key={addr.id || idx}
              className="relative flex flex-col justify-between rounded-xl border border-gray-200 bg-white p-5 shadow-theme-xs dark:border-gray-800 dark:bg-white/2"
            >
              <div>
                <div className="mb-4 flex items-center justify-between border-b border-gray-100 pb-3 dark:border-gray-800">
                  <span className="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
                    {t("profile.address.title")} #{idx + 1}
                  </span>
                  <div className="flex items-center gap-2">
                    {addr.id && (
                      <Link
                        to={`/profile/addresses/${addr.id}/edit`}
                        className="inline-flex size-8 items-center justify-center rounded-lg text-gray-500 hover:bg-brand-50 hover:text-brand-600 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-gray-200 transition-colors"
                        title={t("profile.edit")}
                      >
                        <PencilIcon className="size-4" />
                      </Link>
                    )}
                    {addr.id && (
                      <button
                        onClick={() => handleDelete(addr.id!)}
                        disabled={deletingId === addr.id}
                        className="inline-flex size-8 items-center justify-center rounded-lg text-red-500 hover:bg-red-50 hover:text-red-700 dark:text-red-400 dark:hover:bg-red-900/20 transition-colors"
                        title={t("profile.address.delete")}
                      >
                        <TrashBinIcon className="size-4" />
                      </button>
                    )}
                  </div>
                </div>

                <div className="space-y-3">
                  <div>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      {t("profile.address.street")}
                    </p>
                    <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                      {addr.addressLine1 || "—"}
                    </p>
                  </div>

                  {addr.addressLine2 && (
                    <div>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {t("profile.address.complement")}
                      </p>
                      <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                        {addr.addressLine2}
                      </p>
                    </div>
                  )}

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {t("profile.address.locality")}
                      </p>
                      <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                        {addr.locality || "—"}
                      </p>
                    </div>
                    <div>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {t("profile.address.administrativeArea")}
                      </p>
                      <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                        {addr.administrativeArea || "—"}
                      </p>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {t("profile.address.postalCode")}
                      </p>
                      <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                        {addr.postalCode ? maskCep(addr.postalCode) : "—"}
                      </p>
                    </div>
                    <div>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {t("profile.address.countryCode")}
                      </p>
                      <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                        {addr.countryCode || "—"}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
