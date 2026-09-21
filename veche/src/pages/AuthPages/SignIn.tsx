import { useTranslation } from "react-i18next";
import SignInForm from "@/components/auth/SignInForm";
import PageMeta from "@/components/common/PageMeta";
import AuthLayout from "./AuthPageLayout";

export default function SignIn() {
  const { t } = useTranslation();
  return (
    <>
      <PageMeta
        title={`${t("auth.signInTitle", "Sign In")} | Veche`}
        description={t("auth.signInSubtitle", "Enter your email and password to sign in!")}
      />
      <AuthLayout>
        <SignInForm />
      </AuthLayout>
    </>
  );
}
