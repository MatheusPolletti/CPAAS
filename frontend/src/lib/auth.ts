import api from "@/lib/axios";
import { ErrorType } from "@/types/error-type";
import { LoginFormSchema, RegisterFormSchema } from "@/types/form-schema";
import { RegisterResponse } from "@/types/response-register";
import axios from "axios";
import { signIn } from "next-auth/react";
import z from "zod";

export async function SignIn(
  email: string | null,
  password: string | null,
): Promise<ErrorType> {
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

export async function SignUp(
  username: string | null,
  email: string | null,
  password: string | null,
): Promise<ErrorType> {
  try {
    const error: ErrorType = { status: false };

    const validatedFields = RegisterFormSchema.safeParse({
      username,
      email,
      password,
    });

    if (!validatedFields.success) {
      const tree = z.treeifyError(validatedFields.error);

      if (tree.properties?.username?.errors) {
        error.name = tree.properties?.username?.errors;
      }
      if (tree.properties?.email?.errors) {
        error.email = tree.properties?.email?.errors;
      }
      if (tree.properties?.password?.errors) {
        error.password = tree.properties?.password?.errors;
      }

      if (!error.name && !error.email && !error.password) {
        throw new Error("Dados Inválidos");
      }

      return error;
    }

    const response = await api.post<RegisterResponse>("/auth/register", {
      username,
      email,
      password,
    });

    if (response.data?.success) {
      error.status = true;
      return error;
    }

    error.error = response.data?.message || "Erro ao registrar.";
    return error;
  } catch (err) {
    if (axios.isAxiosError(err)) {
      if (!err.response) {
        return {
          status: false,
          error: "Não foi possível conectar ao servidor.",
        };
      }

      const status = err.response.status;
      if (status === 503 || status === 502 || status === 504) {
        return {
          status: false,
          error: "Servidor indisponível. Tente novamente mais tarde.",
        };
      }

      const message =
        typeof err.response.data?.message === "string"
          ? err.response.data.message
          : "Erro ao tentar realizar o cadastro.";

      return { status: false, error: message };
    }

    return { status: false, error: "Erro desconhecido ao cadastrar." };
  }
}