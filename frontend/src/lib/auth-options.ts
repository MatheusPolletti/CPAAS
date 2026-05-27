import { NextAuthOptions } from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";
import axios from "axios";
import { JWT } from "next-auth/jwt";
import { BACKEND_URL } from "./constant";
import { LoginResponse } from "@/types/response-login";

const inFlightRefreshes = new Map<string, Promise<JWT>>();

async function performRefresh(token: JWT): Promise<JWT> {
  try {
    const response = await axios.post(`${BACKEND_URL}/auth/refresh`, {
      refresh_token: token.refreshToken,
    });

    if (response.data?.success) {
      return {
        ...token,
        accessToken: response.data.data.accessToken,
        refreshToken: response.data.data.refreshToken,
        expiresIn: response.data.data.expiresIn,
        refreshExpiresIn: response.data.data.refreshExpiresIn,
        error: undefined,
      };
    }

    return { ...token, error: "RefreshAccessTokenError" };
  } catch {
    return { ...token, error: "RefreshAccessTokenError" };
  }
}

async function refreshToken(token: JWT): Promise<JWT> {
  const key = token.refreshToken;
  if (!key) return { ...token, error: "RefreshAccessTokenError" };

  const existing = inFlightRefreshes.get(key);
  if (existing) return existing;

  const pending = performRefresh(token).finally(() => {
    inFlightRefreshes.delete(key);
  });
  inFlightRefreshes.set(key, pending);
  return pending;
}

export const authOptions: NextAuthOptions = {
  pages: {
    signIn: "/login",
    signOut: "/login",
  },
  providers: [
    CredentialsProvider({
      name: "Credentials",
      credentials: {
        email: { label: "E-mail", type: "email" },
        password: { label: "Senha", type: "password" },
      },
      async authorize(credentials) {
        try {
          const response = await axios.post<LoginResponse>(
            `${BACKEND_URL}/auth/login`,
            {
              email: credentials?.email,
              password: credentials?.password,
            },
            { headers: { "Content-Type": "application/json" } },
          );

          if (response.status === 200) {
            return {
              id: response.data.data.user.id,
              name: response.data.data.user.username,
              email: response.data.data.user.email,
              accessToken: response.data.data.backendToken.accessToken,
              refreshToken: response.data.data.backendToken.refreshToken,
              expiresIn: response.data.data.backendToken.expiresIn,
              refreshExpiresIn:
                response.data.data.backendToken.refreshExpiresIn,
            };
          }

          throw new Error("E-mail ou senha incorretos.");
        } catch (error) {
          if (axios.isAxiosError(error)) {
            const status = error.response?.status;
            if (status === 401) throw new Error("E-mail ou senha incorretos.");
            if (status === 403) throw new Error("Usuário desativado.");
            if (status === 503 || status === 502 || status === 504)
              throw new Error(
                "Servidor indisponível. Tente novamente mais tarde.",
              );
            if (!error.response)
              throw new Error("Não foi possível conectar ao servidor.");
            throw new Error("Erro ao tentar realizar o login.");
          }

          throw error;
        }
      },
    }),
  ],

  callbacks: {
    async jwt({ token, user, trigger, session }) {
      const now = Date.now();

      if (user) {
        token.id = Number(user.id);
        token.name = user.name;
        token.email = user.email;
        token.accessToken = user.accessToken;
        token.refreshToken = user.refreshToken;
        token.expiresIn = user.expiresIn ? Number(user.expiresIn) : undefined;
        token.refreshExpiresIn = user.refreshExpiresIn
          ? Number(user.refreshExpiresIn)
          : undefined;
        return token;
      }

      if (trigger === "update" && session) {
        token.lastPermissionCheck = now;
        return token;
      }

      if (!token.expiresIn && token.accessToken) {
        try {
          const payload = token.accessToken.split(".")[1];
          const decoded = JSON.parse(
            Buffer.from(payload, "base64").toString("utf-8"),
          ) as { exp?: number };
          if (decoded.exp) {
            token.expiresIn = decoded.exp * 1000;
          }
        } catch {
          // Ignore decode failures and rely on refresh failures to re-login.
        }
      }

      if (token.expiresIn && now >= Number(token.expiresIn) - 10_000) {
        token = await refreshToken(token);
      }

      return token;
    },

    async session({ session, token }) {
      session.user = {
        id: token.id,
        name: token.name as string,
        email: token.email as string,
        accessToken: token.accessToken as string,
        refreshToken: token.refreshToken as string,
        expiresIn: token.expiresIn as number,
        refreshExpiresIn: token.refreshExpiresIn as number,
      };

      if (token.error === "RefreshAccessTokenError") {
        session.error = "RefreshAccessTokenError";
      }

      return session;
    },
  },
};
