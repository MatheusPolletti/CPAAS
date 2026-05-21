import { LoginForm } from "@/components/auth/login-form";

const LoginPage = () => {
  return (
    <main className="relative min-h-svh flex items-center justify-center bg-background bg-[radial-gradient(circle,hsl(var(--muted-foreground)/0.2)_1px,transparent_1px)] bg-size-[28px_28px]">
      <div className="flex flex-col items-center gap-8 w-full max-w-sm px-6 py-10">
        <LoginForm />
      </div>
    </main>
  );
};

export default LoginPage;
