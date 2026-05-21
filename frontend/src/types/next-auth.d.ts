import { DefaultSession } from "next-auth";

declare module "next-auth" {
  interface Session extends DefaultSession {
    user: {
      id: number;
      name: string;
      email: string;
      accessToken: string;
      refreshToken: string;
      expiresIn?: number;
      refreshExpiresIn?: number;
    };
    error?: "RefreshAccessTokenError";
  }

  interface User {
    id: number;
    name: string;
    email: string;
    accessToken: string;
    refreshToken: string;
    expiresIn?: number;
    refreshExpiresIn?: number;
  }
}

declare module "next-auth/jwt" {
  interface JWT {
    id: number;
    name: string;
    email: string;
    roles: string[];
    permissions: string[];
    isTechnician: boolean;
    accessToken: string;
    refreshToken: string;
    expiresIn?: number | string;
    refreshExpiresIn?: number;
    lastPermissionCheck?: number;
    error?: string;
  }
}