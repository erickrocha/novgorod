import { useEffect, useState } from "react";
import type { ChangeEvent, FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { useModal } from "../../hooks/useModal";
import { CalenderIcon, PencilIcon } from "../../icons";
import Input from "../form/input/InputField";
import Label from "../form/Label";
import Button from "../ui/button/Button";
import { Modal } from "../ui/modal";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchProfile, updateUserProfile } from "@/store/authSlice";
import { resourceService } from "@/services/resourceService";
import { Gender } from "@/services/types";
import { maskPhone } from "@/utils/mask";

export default function UserMetaCard() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { user, person } = useAppSelector((state) => state.auth);
  const { isOpen, openModal, closeModal } = useModal();

  const [firstName, setFirstName] = useState("");
  const [surname, setSurname] = useState("");
  const [email, setEmail] = useState("");
  const [phone, setPhone] = useState("");
  const [gender, setGender] = useState("");
  const [dateOfBirth, setDateOfBirth] = useState("");
  const [avatar, setAvatar] = useState<string | null>(null);
  const [previewAvatarUrl, setPreviewAvatarUrl] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!person) {
      dispatch(fetchProfile());
    }
  }, [dispatch, person]);

  useEffect(() => {
    if (isOpen) {
      setFirstName(person?.firstName || user?.name || "");
      setSurname(person?.surname || "");
      setEmail(person?.email || user?.email || "");
      setPhone(maskPhone(person?.phone || ""));
      // Normalize gender to enum if possible
      const g = (person?.gender || "").toUpperCase();
      if (g === Gender.MALE || g === "M" || g === "MASCULINO") {
        setGender(Gender.MALE);
      } else if (g === Gender.FEMALE || g === "F" || g === "FEMININO") {
        setGender(Gender.FEMALE);
      } else {
        setGender(person?.gender || "");
      }
      setDateOfBirth(person?.dateOfBirth || "");
      setAvatar(person?.avatar || null);
      setPreviewAvatarUrl(person?.avatarUrl || null);
      setError(null);
    }
  }, [isOpen, person, user]);

  const handleAvatarChange = async (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    try {
      setIsUploading(true);
      setError(null);
      const res = await resourceService.uploadAvatar(file);
      setAvatar(res.objectKey);
      if (res.cdnUrl) {
        setPreviewAvatarUrl(res.cdnUrl);
      } else {
        setPreviewAvatarUrl(URL.createObjectURL(file));
      }
    } catch (err) {
      console.error("Failed to upload avatar", err);
      setError(t("profile.avatarUploadError"));
    } finally {
      setIsUploading(false);
    }
  };

  const handlePhoneChange = (e: ChangeEvent<HTMLInputElement>) => {
    setPhone(maskPhone(e.target.value));
  };

  const handleSave = async (e: FormEvent) => {
    e.preventDefault();
    if (!firstName.trim()) {
      setError(t("profile.requiredFirstName"));
      return;
    }

    try {
      setIsSaving(true);
      setError(null);
      await dispatch(
        updateUserProfile({
          userId: user?.userId,
          tenantId: user?.tenantId,
          firstName: firstName.trim(),
          surname: surname.trim() || null,
          email: email.trim() || null,
          phone: phone.trim() || null,
          gender: gender.trim() || null,
          dateOfBirth: dateOfBirth || null,
          avatar: avatar || null,
        }),
      ).unwrap();
      closeModal();
    } catch (err: unknown) {
      console.error("Failed to update profile", err);
      setError(typeof err === "string" ? err : t("profile.profileUpdateError"));
    } finally {
      setIsSaving(false);
    }
  };

  const displayName = [person?.firstName || user?.name || "User", person?.surname]
    .filter(Boolean)
    .join(" ");
  const displayEmail = person?.email || user?.email || "—";
  const displayPhone = person?.phone ? maskPhone(person.phone) : "—";
  const displayGender =
    person?.gender?.toUpperCase() === Gender.MALE || person?.gender === "M"
      ? t("profile.genders.male")
      : person?.gender?.toUpperCase() === Gender.FEMALE || person?.gender === "F"
      ? t("profile.genders.female")
      : person?.gender || "—";
  const displayDob = person?.dateOfBirth || "—";
  const currentAvatarUrl = person?.avatarUrl || "/images/user/owner.png";

  return (
    <>
      <div className="mb-6 rounded-2xl border border-gray-200 p-5 lg:p-6 dark:border-gray-800">
        <div className="flex flex-col gap-5 sm:flex-row xl:gap-10">
          <div className="flex-1">
            <div className="mb-6 flex flex-col gap-5 sm:flex-row xl:items-center xl:justify-between">
              <div className="flex w-full flex-col items-start gap-6 sm:flex-row sm:items-center">
                <div className="border-gray-20 size-20 overflow-hidden rounded-full border dark:border-gray-800">
                  <img
                    src={currentAvatarUrl}
                    className="size-20 object-cover"
                    alt="user avatar"
                  />
                </div>
                <div className="text-left">
                  <h4 className="mb-2 text-lg font-semibold text-gray-800 dark:text-white/90">
                    {displayName}
                  </h4>
                  <div className="flex items-center gap-1 sm:gap-3">
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      {user?.role || "User"}
                    </p>
                    {user?.tenantId && (
                      <>
                        <div className="hidden h-3.5 w-px bg-gray-300 sm:block dark:bg-gray-700"></div>
                        <p className="text-sm text-gray-500 dark:text-gray-400">
                          Tenant #{user.tenantId}
                        </p>
                      </>
                    )}
                  </div>
                </div>
              </div>
            </div>
            <div className="relative grid max-w-4xl grid-cols-1 gap-5 sm:grid-cols-2 xl:grid-cols-4 xl:gap-x-11 xl:gap-y-7">
              <div className="w-full">
                <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                  {t("profile.firstName")}
                </p>
                <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                  {person?.firstName || user?.name || "—"}
                </p>
              </div>
              <div className="w-full">
                <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                  {t("profile.surname")}
                </p>
                <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                  {person?.surname || "—"}
                </p>
              </div>
              <div>
                <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                  {t("profile.email")}
                </p>
                <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                  {displayEmail}
                </p>
              </div>
              <div>
                <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                  {t("profile.phone")}
                </p>
                <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                  {displayPhone}
                </p>
              </div>
              <div>
                <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                  {t("profile.gender")}
                </p>
                <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                  {displayGender}
                </p>
              </div>
              <div>
                <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                  {t("profile.dateOfBirth")}
                </p>
                <p className="text-sm font-medium text-gray-800 dark:text-white/90">
                  {displayDob}
                </p>
              </div>
            </div>
          </div>
          <div>
            <button
              onClick={openModal}
              className="flex h-10 w-full items-center justify-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-2.5 text-sm font-medium text-gray-700 shadow-theme-xs hover:bg-gray-50 hover:text-gray-800 lg:inline-flex lg:w-auto dark:border-gray-700 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-white/3 dark:hover:text-gray-200"
            >
              <PencilIcon className="size-4.5" />
              {t("profile.edit")}
            </button>
          </div>
        </div>
      </div>

      <Modal isOpen={isOpen} onClose={closeModal} className="max-w-[700px]">
        <div className="relative no-scrollbar w-full max-w-[700px] overflow-y-auto rounded-3xl bg-white p-4 lg:p-11 dark:bg-gray-900">
          <div className="px-2 pr-14">
            <h4 className="mb-2 text-2xl font-semibold text-gray-800 dark:text-white/90">
              {t("profile.editProfile")}
            </h4>
            <p className="mb-6 text-sm text-gray-500 lg:mb-7 dark:text-gray-400">
              {t("profile.editProfileDesc")}
            </p>
            {error && (
              <div className="mb-4 rounded-lg bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
                {error}
              </div>
            )}
          </div>
          <form onSubmit={handleSave} className="flex flex-col">
            <div className="custom-scrollbar h-[450px] overflow-y-auto px-2 pb-3">
              <div>
                <h4 className="mb-6 text-lg font-medium text-gray-800 dark:text-white/90">
                  {t("profile.avatar")}
                </h4>
                <div className="mb-6 flex max-w-sm items-center gap-6 lg:pr-5">
                  <div className="relative size-20 shrink-0 rounded-full sm:size-25">
                    <img
                      src={previewAvatarUrl || currentAvatarUrl}
                      alt="Profile Picture"
                      className="size-20 rounded-full object-cover sm:size-25"
                    />
                    <label
                      htmlFor="avatar-file-upload"
                      className="absolute right-0 bottom-0 flex size-8 cursor-pointer items-center justify-center rounded-full border border-gray-200 bg-white text-gray-500 hover:bg-gray-100 dark:border-gray-800 dark:bg-gray-900 dark:text-gray-400 dark:hover:bg-gray-800"
                    >
                      <input
                        type="file"
                        id="avatar-file-upload"
                        accept="image/*"
                        onChange={handleAvatarChange}
                        disabled={isUploading}
                        className="hidden"
                      />
                      <svg
                        width="20"
                        height="20"
                        viewBox="0 0 20 20"
                        fill="none"
                        xmlns="http://www.w3.org/2000/svg"
                      >
                        <path
                          d="M12.6731 3.41904C12.4371 3.10308 12.0659 2.91699 11.6715 2.91699H8.32809C7.93374 2.91699 7.56252 3.10308 7.32656 3.41904L6.83173 4.08164C6.59576 4.3976 6.22454 4.58369 5.83019 4.58369H3.5415C2.85115 4.58369 2.2915 5.14333 2.2915 5.83369V14.3754C2.2915 15.0657 2.85115 15.6254 3.5415 15.6254H16.4582C17.1485 15.6254 17.7082 15.0657 17.7082 14.3754V5.83369C17.7082 5.14333 17.1485 4.58369 16.4582 4.58369H14.1694C13.7751 4.58369 13.4039 4.3976 13.1679 4.08164L12.6731 3.41904Z"
                          stroke="currentColor"
                          strokeWidth="1.5"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                        <path
                          d="M13.3332 9.79362C13.3332 11.6346 11.8408 13.127 9.99984 13.127C8.15889 13.127 6.6665 11.6346 6.6665 9.79362C6.6665 7.95267 8.15889 6.46029 9.99984 6.46029C11.8408 6.46029 13.3332 7.95267 13.3332 9.79362Z"
                          stroke="currentColor"
                          strokeWidth="1.5"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                      </svg>
                    </label>
                  </div>
                  <div>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      {isUploading
                        ? t("profile.saving")
                        : t("profile.editProfileDesc")}
                    </p>
                  </div>
                </div>
              </div>

              <div className="my-7">
                <h5 className="mb-5 text-lg font-medium text-gray-800 lg:mb-6 dark:text-white/90">
                  {t("profile.personalInfo")}
                </h5>

                <div className="grid grid-cols-1 gap-x-6 gap-y-5 lg:grid-cols-2">
                  <div className="col-span-2 lg:col-span-1">
                    <Label>{t("profile.firstName")} *</Label>
                    <Input
                      type="text"
                      value={firstName}
                      onChange={(e) => setFirstName(e.target.value)}
                      placeholder={t("profile.firstName")}
                    />
                  </div>

                  <div className="col-span-2 lg:col-span-1">
                    <Label>{t("profile.surname")}</Label>
                    <Input
                      type="text"
                      value={surname}
                      onChange={(e) => setSurname(e.target.value)}
                      placeholder={t("profile.surname")}
                    />
                  </div>

                  <div className="col-span-2 lg:col-span-1">
                    <Label>{t("profile.email")}</Label>
                    <Input
                      type="email"
                      value={email}
                      onChange={(e) => setEmail(e.target.value)}
                      placeholder={t("profile.email")}
                    />
                  </div>

                  <div className="col-span-2 lg:col-span-1">
                    <Label>{t("profile.phone")}</Label>
                    <Input
                      type="tel"
                      value={phone}
                      onChange={handlePhoneChange}
                      placeholder="(99) 99999-9999"
                    />
                  </div>

                  <div className="col-span-2 lg:col-span-1">
                    <Label>{t("profile.gender")}</Label>
                    <select
                      value={gender}
                      onChange={(e) => setGender(e.target.value)}
                      className="h-11 w-full appearance-none rounded-lg border border-gray-300 bg-transparent px-4 py-2.5 text-sm text-gray-800 shadow-theme-xs focus:border-brand-300 focus:outline-hidden dark:border-gray-700 dark:bg-gray-900 dark:text-white/90 dark:focus:border-brand-800"
                    >
                      <option value="">{t("profile.selectGender")}</option>
                      <option value={Gender.MALE}>{t("profile.genders.male")}</option>
                      <option value={Gender.FEMALE}>{t("profile.genders.female")}</option>
                    </select>
                  </div>

                  <div className="col-span-2 lg:col-span-1">
                    <Label>{t("profile.dateOfBirth")}</Label>
                    <div className="relative">
                      <input
                        type="date"
                        value={dateOfBirth}
                        onChange={(e) => setDateOfBirth(e.target.value)}
                        className="h-11 w-full appearance-none rounded-lg border border-gray-300 bg-transparent py-2.5 ps-4 pe-10 text-start text-sm text-gray-800 shadow-theme-xs placeholder:text-gray-400 focus:border-brand-300 focus:ring-3 focus:ring-brand-500/20 focus:outline-hidden dark:border-gray-700 dark:bg-gray-900 dark:text-white/90 dark:placeholder:text-white/30 dark:focus:border-brand-800"
                      />
                      <span className="pointer-events-none absolute inset-e-3 top-1/2 -translate-y-1/2 text-gray-500 dark:text-gray-400">
                        <CalenderIcon className="size-5" />
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div className="mt-6 flex items-center gap-3 px-2 lg:justify-end">
              <Button
                size="sm"
                variant="outline"
                type="button"
                onClick={closeModal}
                disabled={isSaving || isUploading}
              >
                {t("profile.close")}
              </Button>
              <Button
                size="sm"
                type="submit"
                disabled={isSaving || isUploading}
              >
                {isSaving ? t("profile.saving") : t("profile.saveChanges")}
              </Button>
            </div>
          </form>
        </div>
      </Modal>
    </>
  );
}

