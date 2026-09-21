import { useEffect } from "react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { PencilIcon } from "../../icons";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchProfile } from "@/store/authSlice";
import { Gender } from "@/services/types";
import { maskPhone } from "@/utils/mask";

export default function UserMetaCard() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { user, person } = useAppSelector((state) => state.auth);

  useEffect(() => {
    if (!person) {
      dispatch(fetchProfile());
    }
  }, [dispatch, person]);

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
    <div className="mb-6 rounded-2xl border border-gray-200 bg-white p-5 shadow-theme-xs lg:p-6 dark:border-gray-800 dark:bg-white/[0.03]">
      <div className="flex flex-col gap-5 sm:flex-row xl:gap-10">
        <div className="flex-1">
          <div className="mb-6 flex flex-col gap-5 sm:flex-row xl:items-center xl:justify-between">
            <div className="flex w-full flex-col items-start gap-6 sm:flex-row sm:items-center">
              <div className="border-gray-200 size-20 overflow-hidden rounded-full border dark:border-gray-800">
                <img
                  src={currentAvatarUrl}
                  className="size-20 object-cover"
                  alt="user avatar"
                />
              </div>
              <div className="text-left">
                <h4 className="mb-2 text-lg font-semibold text-gray-900 dark:text-white/90">
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
              <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                {person?.firstName || user?.name || "—"}
              </p>
            </div>
            <div className="w-full">
              <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                {t("profile.surname")}
              </p>
              <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                {person?.surname || "—"}
              </p>
            </div>
            <div>
              <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                {t("profile.email")}
              </p>
              <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                {displayEmail}
              </p>
            </div>
            <div>
              <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                {t("profile.phone")}
              </p>
              <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                {displayPhone}
              </p>
            </div>
            <div>
              <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                {t("profile.gender")}
              </p>
              <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                {displayGender}
              </p>
            </div>
            <div>
              <p className="mb-2 text-xs leading-normal text-gray-500 dark:text-gray-400">
                {t("profile.dateOfBirth")}
              </p>
              <p className="text-sm font-medium text-gray-900 dark:text-white/90">
                {displayDob}
              </p>
            </div>
          </div>
        </div>
        <div>
          <Link
            to="/profile/edit"
            className="flex h-10 w-full items-center justify-center gap-2 rounded-lg border border-gray-200 bg-white px-4 py-2.5 text-sm font-medium text-gray-900 shadow-theme-xs hover:bg-brand-50 hover:text-brand-600 hover:border-brand-500 lg:inline-flex lg:w-auto dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-brand-950 dark:hover:text-brand-400 transition-all duration-300"
          >
            <PencilIcon className="size-4.5" />
            {t("profile.edit")}
          </Link>
        </div>
      </div>
    </div>
  );
}
