"use client";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { SyntheticEvent, useRef, useState } from "react";
import { ArrowRight, Eye, EyeClosed } from "lucide-react";
import { toast } from "sonner";
import { SignUp } from "@/lib/auth";
import { ErrorType } from "@/types/error-type";
import { motion } from "framer-motion";
import Link from "next/link";

export function RegisterForm({
  className,
  ...props
}: React.ComponentProps<"form">) {
  const [showPassword, setShowPassword] = useState(false);
  const [registering, setRegistering] = useState(false);
  const [error, setError] = useState<ErrorType>({ status: false });
  const passwordRef = useRef<HTMLInputElement>(null);

  const handleTogglePassword = () => {
    setShowPassword(!showPassword);

    if (passwordRef.current) {
      passwordRef.current.focus();
      const length = passwordRef.current.value.length;
      passwordRef.current.setSelectionRange(length, length);
    }
  };

  const handleSubmit = async (e: SyntheticEvent<HTMLFormElement>) => {
    e.preventDefault();
    setRegistering(true);

    try {
      setError({ status: false });

      const form = e.currentTarget as HTMLFormElement;
      const formData = new FormData(form);

      const response = await SignUp(
        formData.get("user-name") as string,
        formData.get("user-identifier") as string,
        formData.get("user-secret") as string,
      );

      if (response?.status) {
        window.location.href = "/login";
        return;
      }

      if (response?.name || response?.email || response?.password) {
        setError(response);
        return;
      }

      if (response?.error) {
        const msg = response.error.toLowerCase();

        if (
          msg.includes("indisponível") ||
          msg.includes("conexão") ||
          msg.includes("conexao") ||
          msg.includes("servidor") ||
          msg.includes("unknown")
        ) {
          throw new Error(response.error);
        }

        if (
          msg.includes("nome") ||
          msg.includes("usuário") ||
          msg.includes("usuario")
        ) {
          setError({ status: false, name: [response.error] });
        } else if (msg.includes("email") || msg.includes("e-mail")) {
          setError({ status: false, email: [response.error] });
        } else if (msg.includes("senha") || msg.includes("password")) {
          setError({ status: false, password: [response.error] });
        } else {
          throw new Error(response.error);
        }
        return;
      }
    } catch (err) {
      if (err instanceof Error) {
        toast.error(err.message, {
          position: "top-center",
        });
      } else {
        toast.error("Ocorreu um erro desconhecido.", {
          position: "top-center",
        });
      }
    } finally {
      setRegistering(false);
    }
  };

  return (
    <>
      <div className="flex flex-col items-center gap-4">
        <motion.p
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2 }}
          className="text-lg font-semibold text-black mt-2"
        >
          Crie sua conta para continuar
        </motion.p>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.3 }}
        className="w-full rounded-2xl bg-card border border-border shadow-md px-8 py-8"
      >
        <form
          onSubmit={handleSubmit}
          autoComplete="off"
          className={cn("flex flex-col gap-5", className)}
          {...props}
        >
          <input
            type="text"
            name="username"
            autoComplete="username"
            tabIndex={-1}
            aria-hidden="true"
            className="hidden"
            readOnly
          />
          <input
            type="password"
            name="password"
            autoComplete="new-password"
            tabIndex={-1}
            aria-hidden="true"
            className="hidden"
            readOnly
          />

          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.4 }}
            className="flex flex-col gap-1.5"
          >
            <label
              htmlFor="user-name"
              className="text-sm font-medium text-foreground"
            >
              Nome
            </label>
            <Input
              id="user-name"
              name="user-name"
              type="text"
              placeholder="Seu nome"
              required
              autoComplete="off"
              readOnly
              onFocus={(e) => e.currentTarget.removeAttribute("readonly")}
              data-lpignore="true"
              data-form-type="other"
              data-1p-ignore
              className={`h-12 rounded-xl bg-muted border-border transition-colors duration-200
								focus-visible:ring-0 focus-visible:border-blue-500 focus-visible:border-2
								focus-visible:bg-background ${error.name ? "border-red-500" : ""}`}
              onInvalid={(e) =>
                (e.target as HTMLInputElement).setCustomValidity(
                  "Por favor, insira seu nome.",
                )
              }
              onInput={(e) =>
                (e.target as HTMLInputElement).setCustomValidity("")
              }
            />
            {error?.name && (
              <motion.ul
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.3 }}
                className="mt-0.5"
              >
                {error.name.map((msg, index) => (
                  <li key={index} className="text-sm text-red-500">
                    {msg}
                  </li>
                ))}
              </motion.ul>
            )}
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.5 }}
            className="flex flex-col gap-1.5"
          >
            <label
              htmlFor="user-identifier"
              className="text-sm font-medium text-foreground"
            >
              E-mail
            </label>
            <Input
              id="user-identifier"
              name="user-identifier"
              type="email"
              placeholder="Email"
              required
              autoComplete="off"
              readOnly
              onFocus={(e) => e.currentTarget.removeAttribute("readonly")}
              data-lpignore="true"
              data-form-type="other"
              data-1p-ignore
              className={`h-12 rounded-xl bg-muted border-border transition-colors duration-200
								focus-visible:ring-0 focus-visible:border-blue-500 focus-visible:border-2
								focus-visible:bg-background ${error.email ? "border-red-500" : ""}`}
              onInvalid={(e) =>
                (e.target as HTMLInputElement).setCustomValidity(
                  "Por favor, insira um e-mail válido.",
                )
              }
              onInput={(e) =>
                (e.target as HTMLInputElement).setCustomValidity("")
              }
            />
            {error?.email && (
              <motion.ul
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.3 }}
                className="mt-0.5"
              >
                {error.email.map((msg, index) => (
                  <li key={index} className="text-sm text-red-500">
                    {msg}
                  </li>
                ))}
              </motion.ul>
            )}
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.6 }}
            className="flex flex-col gap-1.5"
          >
            <label
              htmlFor="user-secret"
              className="text-sm font-medium text-foreground"
            >
              Senha
            </label>
            <div className="relative">
              <Input
                id="user-secret"
                name="user-secret"
                ref={passwordRef}
                type={showPassword ? "text" : "password"}
                placeholder="••••••••"
                required
                autoComplete="new-password"
                readOnly
                onFocus={(e) => e.currentTarget.removeAttribute("readonly")}
                data-lpignore="true"
                data-form-type="other"
                data-1p-ignore
                className={`h-12 rounded-xl bg-muted border-border transition-colors duration-200
									focus-visible:ring-0 focus-visible:border-blue-500 focus-visible:border-2
									focus-visible:bg-background pr-10 ${error.password ? "border-red-500" : ""}`}
              />
              <button
                type="button"
                onClick={handleTogglePassword}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground cursor-pointer"
                tabIndex={-1}
              >
                {showPassword ? <Eye size={18} /> : <EyeClosed size={18} />}
              </button>
            </div>
            {error?.password && (
              <motion.ul
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.3 }}
                className="mt-0.5"
              >
                {error.password.map((msg, index) => (
                  <li key={index} className="text-sm text-red-500">
                    {msg}
                  </li>
                ))}
              </motion.ul>
            )}
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.5, delay: 0.6 }}
            className="flex flex-col gap-1.5"
          >
            <p className="text-sm text-muted-foreground">
              Já tem uma conta?{" "}
              <Link href="/login" className="text-blue-500 hover:underline">
                {" "}
                Faça login{" "}
              </Link>
            </p>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5, delay: 0.7 }}
          >
            <Button
              type="submit"
              disabled={registering}
              className="w-full h-12 rounded-xl bg-blue-500 hover:bg-blue-500/80 dark:bg-blue-700 text-white font-semibold text-base mt-1 flex items-center justify-center gap-2"
            >
              {registering ? "Criando conta..." : "Criar conta"}
              {!registering && <ArrowRight size={18} />}
            </Button>
          </motion.div>
        </form>
      </motion.div>
    </>
  );
}
