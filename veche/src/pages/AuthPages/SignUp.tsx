import { useTranslation } from "react-i18next";
import SignUpForm from "@/components/auth/SignUpForm";
import PageMeta from "@/components/common/PageMeta";
import AuthLayout from "./AuthPageLayout";

export default function SignUp() {
  const { t } = useTranslation();
  return (
    <>
      <PageMeta
        title={`${t("auth.signUpTitle", "Sign Up")} | Veche`}
        description={t("auth.signUpSubtitle", "Enter your email and password to sign up!")}
      />
      <AuthLayout>
        <SignUpForm />
      </AuthLayout>
    </>
  );
}
