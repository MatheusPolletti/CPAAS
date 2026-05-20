import { ErrorType } from "@/types/error-type";
import { LoginFormSchema } from "@/types/form-schema";
import { signIn } from "next-auth/react";
import z from "zod";

export async function SignIn(email: string | null, password: string | null) {
  try {
    const error: ErrorType = { status: false };

    const validatedFields = LoginFormSchema.safeParse({
      email,
      password,
    });

    if (!validatedFields.success) {
      const tree = z.treeifyError(validatedFields.error);

      if (tree.properties?.email?.errors) {
        error.email = tree.properties?.email?.errors;
      }
      if (tree.properties?.password?.errors) {
        error.password = tree.properties?.password?.errors;
      }

      if (!error.email && !error.password) {
        throw new Error("Dados Inválidos");
      }

      return error;
    }

    const result = await signIn("credentials", {
      email,
      password,
      redirect: false,
    });

    if (result?.ok) {
      error.status = true;
    } else {
      error.status = false;
      error.error = result?.error || "Erro ao logar na conta.";
    }

    return error;
  } catch {
    return { status: false, error: "Erro desconhecido ao logar." };
  }
}