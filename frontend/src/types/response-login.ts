export interface UserProfile {
  id: number;
  username: string;
  email: string;
}

export interface BackendToken {
  accessToken: string;
  expiresIn: number;
  refreshToken: string;
  refreshExpiresIn: number;
}

export interface LoginResponse {
    user: UserProfile;
    backendToken: BackendToken;
}
