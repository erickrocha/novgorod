import { useEffect, useState } from "react";
import type { ChangeEvent, FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { useModal } from "../../hooks/useModal";
import { PencilIcon, PlusIcon, TrashBinIcon } from "../../icons";
import Input from "../form/input/InputField";
import Label from "../form/Label";
import Button from "../ui/button/Button";
import { Modal } from "../ui/modal";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
  addPersonAddress,
  deletePersonAddress,
  fetchPersonAddresses,
  updatePersonAddress,
} from "@/store/authSlice";
import type { PersonAddress } from "@/services/types";
import { maskCep } from "@/utils/mask";

export default function UserAddressCard() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { person, addresses } = useAppSelector((state) => state.auth);
  const { isOpen, openModal, closeModal } = useModal();

  const [isEdit, setIsEdit] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [postalCode, setPostalCode] = useState("");
  const [addressLine1, setAddressLine1] = useState("");
  const [addressLine2, setAddressLine2] = useState("");
  const [locality, setLocality] = useState("");
  const [administrativeArea, setAdministrativeArea] = useState("");
  const [countryCode, setCountryCode] = useState("BR");
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<number | null>(null);

  useEffect(() => {
    if (person?.id && addresses.length === 0) {
      dispatch(fetchPersonAddresses(person.id));
    }
  }, [dispatch, person?.id, addresses.length]);

  const handleOpenAdd = () => {
    setIsEdit(false);
    setEditingId(null);
    setPostalCode("");
    setAddressLine1("");
    setAddressLine2("");
    setLocality("");
    setAdministrativeArea("");
    setCountryCode("BR");
    setError(null);
    openModal();
  };

  const handleOpenEdit = (addr: PersonAddress) => {
    setIsEdit(true);
    setEditingId(addr.id ?? null);
    setPostalCode(maskCep(addr.postalCode || ""));
    setAddressLine1(addr.addressLine1 || "");
    setAddressLine2(addr.addressLine2 || "");
    setLocality(addr.locality || "");
    setAdministrativeArea(addr.administrativeArea || "");
    setCountryCode(addr.countryCode || "BR");
    setError(null);
    openModal();
  };

  const handlePostalCodeChange = (e: ChangeEvent<HTMLInputElement>) => {
    setPostalCode(maskCep(e.target.value));
  };

  const handleSave = async (e: FormEvent) => {
    e.preventDefault();
    if (!person?.id) {
      setError(t("profile.profileUpdateError"));
      return;
    }

    try {
      setIsSaving(true);
      setError(null);

      const input = {
        tenantId: person.tenantId,
        personId: person.id,
        postalCode: postalCode.trim() || null,
        addressLine1: addressLine1.trim() || null,
        addressLine2: addressLine2.trim() || null,
        locality: locality.trim() || null,
        administrativeArea: administrativeArea.trim() || null,
        countryCode: countryCode.trim().toUpperCase() || null,
      };

      if (isEdit && editingId) {
        await dispatch(
          updatePersonAddress({ id: editingId, input })
        ).unwrap();
      } else {
        await dispatch(addPersonAddress(input)).unwrap();
      }

      closeModal();
    } catch (err: unknown) {
      console.error("Failed to save address", err);
      setError(typeof err === "string" ? err : t("profile.profileUpdateError"));
    } finally {
      setIsSaving(false);
    }
  };

  const handleDelete = async (id: number) => {
    if (!window.confirm(t("profile.address.confirmDelete"))) {
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
    <>
      <div className="rounded-2xl border border-gray-200 p-5 lg:p-6 dark:border-gray-800">
        <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <h4 className="text-lg font-semibold text-gray-800 dark:text-white/90">
            {t("profile.address.title")}
          </h4>
          <Button
            size="sm"
            onClick={handleOpenAdd}
            className="inline-flex items-center gap-2"
          >
            <PlusIcon className="size-4" />
            {t("profile.address.addAddress")}
          </Button>
        </div>

        {addresses.length === 0 ? (
          <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-gray-300 py-10 text-center dark:border-gray-700">
            <p className="mb-4 text-sm text-gray-500 dark:text-gray-400">
              {t("profile.address.noAddresses")}
            </p>
            <Button
              size="sm"
              variant="outline"
              onClick={handleOpenAdd}
              className="inline-flex items-center gap-2"
            >
              <PlusIcon className="size-4" />
              {t("profile.address.addAddress")}
            </Button>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            {addresses.map((addr, idx) => (
              <div
                key={addr.id || idx}
                className="relative flex flex-col justify-between rounded-xl border border-gray-200 p-5 dark:border-gray-800 dark:bg-white/2"
              >
                <div>
                  <div className="mb-4 flex items-center justify-between border-b border-gray-100 pb-3 dark:border-gray-800">
                    <span className="text-xs font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500">
                      {t("profile.address.title")} #{idx + 1}
                    </span>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleOpenEdit(addr)}
                        className="inline-flex size-8 items-center justify-center rounded-lg text-gray-500 hover:bg-gray-100 hover:text-gray-800 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-gray-200"
                        title={t("profile.edit")}
                      >
                        <PencilIcon className="size-4" />
                      </button>
                      {addr.id && (
                        <button
                          onClick={() => handleDelete(addr.id!)}
                          disabled={deletingId === addr.id}
                          className="inline-flex size-8 items-center justify-center rounded-lg text-red-500 hover:bg-red-50 hover:text-red-700 dark:text-red-400 dark:hover:bg-red-900/20"
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
                      <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                        {addr.addressLine1 || "—"}
                      </p>
                    </div>

                    {addr.addressLine2 && (
                      <div>
                        <p className="text-xs text-gray-500 dark:text-gray-400">
                          {t("profile.address.complement")}
                        </p>
                        <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                          {addr.addressLine2}
                        </p>
                      </div>
                    )}

                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <p className="text-xs text-gray-500 dark:text-gray-400">
                          {t("profile.address.locality")}
                        </p>
                        <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                          {addr.locality || "—"}
                        </p>
                      </div>
                      <div>
                        <p className="text-xs text-gray-500 dark:text-gray-400">
                          {t("profile.address.administrativeArea")}
                        </p>
                        <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                          {addr.administrativeArea || "—"}
                        </p>
                      </div>
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <p className="text-xs text-gray-500 dark:text-gray-400">
                          {t("profile.address.postalCode")}
                        </p>
                        <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                          {addr.postalCode ? maskCep(addr.postalCode) : "—"}
                        </p>
                      </div>
                      <div>
                        <p className="text-xs text-gray-500 dark:text-gray-400">
                          {t("profile.address.countryCode")}
                        </p>
                        <p className="text-sm font-medium text-gray-800 dark:text-white/90">
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

      <Modal isOpen={isOpen} onClose={closeModal} className="max-w-[700px]">
        <div className="relative no-scrollbar w-full overflow-y-auto rounded-3xl bg-white p-4 lg:p-11 dark:bg-gray-900">
          <div className="px-2 pr-14">
            <h4 className="mb-2 text-2xl font-semibold text-gray-800 dark:text-white/90">
              {isEdit
                ? t("profile.address.editAddress")
                : t("profile.address.addAddress")}
            </h4>
            <p className="mb-6 text-sm text-gray-500 lg:mb-7 dark:text-gray-400">
              {isEdit
                ? t("profile.address.editAddressDesc")
                : t("profile.address.addAddressDesc")}
            </p>
            {error && (
              <div className="mb-4 rounded-lg bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
                {error}
              </div>
            )}
          </div>
          <form onSubmit={handleSave} className="flex flex-col">
            <div className="custom-scrollbar overflow-y-auto px-2">
              <div className="grid grid-cols-1 gap-x-6 gap-y-5 lg:grid-cols-2">
                <div>
                  <Label>{t("profile.address.postalCode")}</Label>
                  <Input
                    type="text"
                    value={postalCode}
                    onChange={handlePostalCodeChange}
                    placeholder={t("profile.address.postalCodePlaceholder")}
                  />
                </div>

                <div>
                  <Label>{t("profile.address.countryCode")}</Label>
                  <Input
                    type="text"
                    value={countryCode}
                    onChange={(e) => setCountryCode(e.target.value.toUpperCase())}
                    placeholder={t("profile.address.countryPlaceholder")}
                  />
                </div>

                <div className="col-span-1 lg:col-span-2">
                  <Label>{t("profile.address.street")}</Label>
                  <Input
                    type="text"
                    value={addressLine1}
                    onChange={(e) => setAddressLine1(e.target.value)}
                    placeholder={t("profile.address.streetPlaceholder")}
                  />
                </div>

                <div className="col-span-1 lg:col-span-2">
                  <Label>{t("profile.address.complement")}</Label>
                  <Input
                    type="text"
                    value={addressLine2}
                    onChange={(e) => setAddressLine2(e.target.value)}
                    placeholder={t("profile.address.complementPlaceholder")}
                  />
                </div>

                <div>
                  <Label>{t("profile.address.locality")}</Label>
                  <Input
                    type="text"
                    value={locality}
                    onChange={(e) => setLocality(e.target.value)}
                    placeholder={t("profile.address.localityPlaceholder")}
                  />
                </div>

                <div>
                  <Label>{t("profile.address.administrativeArea")}</Label>
                  <Input
                    type="text"
                    value={administrativeArea}
                    onChange={(e) => setAdministrativeArea(e.target.value)}
                    placeholder={t("profile.address.administrativeAreaPlaceholder")}
                  />
                </div>
              </div>
            </div>
            <div className="mt-6 flex items-center gap-3 px-2 lg:justify-end">
              <Button
                size="sm"
                variant="outline"
                type="button"
                onClick={closeModal}
                disabled={isSaving}
              >
                {t("profile.close")}
              </Button>
              <Button size="sm" type="submit" disabled={isSaving}>
                {isSaving
                  ? t("profile.address.saving")
                  : t("profile.address.saveAddress")}
              </Button>
            </div>
          </form>
        </div>
      </Modal>
    </>
  );
}
