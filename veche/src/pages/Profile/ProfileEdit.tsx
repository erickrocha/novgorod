import { useEffect, useState } from "react";
import type { ChangeEvent, FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import PageBreadcrumb from "@/components/common/PageBreadCrumb";
import PageMeta from "@/components/common/PageMeta";
import ComponentCard from "@/components/common/ComponentCard";
import Input from "@/components/form/input/InputField";
import Label from "@/components/form/Label";
import Button from "@/components/ui/button/Button";
import { CalenderIcon } from "@/icons";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchProfile, updateUserProfile } from "@/store/authSlice";
import { resourceService } from "@/services/resourceService";
import { Gender } from "@/services/types";
import { maskPhone } from "@/utils/mask";

export default function ProfileEdit() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { user, person } = useAppSelector((state) => state.auth);

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
    if (person || user) {
      queueMicrotask(() => {
        setFirstName(person?.firstName || user?.name || "");
        setSurname(person?.surname || "");
        setEmail(person?.email || user?.email || "");
        setPhone(maskPhone(person?.phone || ""));

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
      });
    }
  }, [person, user]);

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
      setError(t("profile.avatarUploadError", "Falha ao enviar avatar"));
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
      setError(t("profile.requiredFirstName", "O primeiro nome é obrigatório."));
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
      navigate("/profile");
    } catch (err: unknown) {
      console.error("Failed to update profile", err);
      setError(typeof err === "string" ? err : t("profile.profileUpdateError", "Erro ao atualizar perfil."));
    } finally {
      setIsSaving(false);
    }
  };

  const currentAvatarUrl = person?.avatarUrl || "/images/user/owner.png";

  return (
    <>
      <PageMeta
        title={`${t("profile.editProfile", "Editar Perfil")} | Veche`}
        description="Edit personal details and avatar"
      />
      <PageBreadcrumb
        pageTitle={t("profile.editProfile", "Editar Perfil")}
        items={[{ label: t("profile.title", "Perfil"), href: "/profile" }]}
      />
      <ComponentCard title={t("profile.editProfile", "Editar Perfil")}>
        {error && (
          <div className="mb-5 rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-600 dark:bg-error-500/10 dark:border-error-500/20">
            {error}
          </div>
        )}

        <form onSubmit={handleSave} className="space-y-6">
          {/* Avatar Section */}
          <div>
            <Label className="mb-3 block font-semibold text-gray-700 dark:text-gray-300">
              {t("profile.avatar", "Avatar")}
            </Label>
            <div className="flex items-center gap-6">
              <div className="relative size-24 shrink-0 rounded-full border border-gray-200 dark:border-gray-800">
                <img
                  src={previewAvatarUrl || currentAvatarUrl}
                  alt="Profile Picture"
                  className="size-24 rounded-full object-cover"
                />
                <label
                  htmlFor="profile-avatar-upload"
                  className="absolute right-0 bottom-0 flex size-8 cursor-pointer items-center justify-center rounded-full border border-gray-200 bg-white text-gray-500 shadow-sm hover:bg-gray-100 dark:border-gray-800 dark:bg-gray-900 dark:text-gray-400 dark:hover:bg-gray-800 transition-colors"
                >
                  <input
                    type="file"
                    id="profile-avatar-upload"
                    accept="image/*"
                    onChange={handleAvatarChange}
                    disabled={isUploading || isSaving}
                    className="hidden"
                  />
                  <svg
                    width="18"
                    height="18"
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
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {isUploading
                  ? t("profile.saving", "Enviando imagem...")
                  : t("profile.editProfileDesc", "Atualize seus dados para manter seu perfil atualizado.")}
              </p>
            </div>
          </div>

          {/* Personal Info Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            <div>
              <Label htmlFor="firstName">
                {t("profile.firstName", "Nome")} *
              </Label>
              <Input
                id="firstName"
                type="text"
                required
                value={firstName}
                onChange={(e) => setFirstName(e.target.value)}
                placeholder={t("profile.firstName", "Nome")}
              />
            </div>

            <div>
              <Label htmlFor="surname">
                {t("profile.surname", "Sobrenome")}
              </Label>
              <Input
                id="surname"
                type="text"
                value={surname}
                onChange={(e) => setSurname(e.target.value)}
                placeholder={t("profile.surname", "Sobrenome")}
              />
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            <div>
              <Label htmlFor="profileEmail">
                {t("profile.email", "E-mail")}
              </Label>
              <Input
                id="profileEmail"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder={t("profile.email", "cliente@exemplo.com")}
              />
            </div>

            <div>
              <Label htmlFor="profilePhone">
                {t("profile.phone", "Telefone")}
              </Label>
              <Input
                id="profilePhone"
                type="tel"
                value={phone}
                onChange={handlePhoneChange}
                placeholder="(00) 00000-0000"
              />
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            <div>
              <Label htmlFor="profileGender">
                {t("profile.gender", "Gênero")}
              </Label>
              <select
                id="profileGender"
                value={gender}
                onChange={(e) => setGender(e.target.value)}
                className="h-11 w-full appearance-none rounded-lg border border-gray-300 bg-transparent px-4 py-2.5 text-sm text-gray-800 shadow-theme-xs focus:border-brand-300 focus:outline-hidden dark:border-gray-700 dark:bg-gray-900 dark:text-white/90 dark:focus:border-brand-800"
              >
                <option value="">{t("profile.selectGender", "Selecione o gênero")}</option>
                <option value={Gender.MALE}>{t("profile.genders.male", "Masculino")}</option>
                <option value={Gender.FEMALE}>{t("profile.genders.female", "Feminino")}</option>
              </select>
            </div>

            <div>
              <Label htmlFor="dateOfBirth">
                {t("profile.dateOfBirth", "Data de Nascimento")}
              </Label>
              <div className="relative">
                <input
                  id="dateOfBirth"
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

          {/* Actions: Cancel on left, Save on right */}
          <div className="flex items-center justify-between pt-4 border-t border-gray-100 dark:border-gray-800">
            <Link to="/profile">
              <Button variant="outline" type="button">
                {t("profile.cancel", "Cancelar")}
              </Button>
            </Link>
            <Button disabled={isSaving || isUploading} type="submit">
              {isSaving
                ? t("profile.saving", "Salvando...")
                : t("profile.saveChanges", "Salvar Alterações")}
            </Button>
          </div>
        </form>
      </ComponentCard>
    </>
  );
}
