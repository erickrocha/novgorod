import SignInForm from "@/components/auth/SignInForm";
import PageMeta from "@/components/common/PageMeta";
import AuthLayout from "./AuthPageLayout";

export default function SignIn() {
  return (
    <>
      <PageMeta
        title="Sign In | Kremlin"
        description="Sign in to Kremlin - Novgorod admin e-commerce"
      />
      <AuthLayout>
        <SignInForm />
      </AuthLayout>
    </>
  );
}
