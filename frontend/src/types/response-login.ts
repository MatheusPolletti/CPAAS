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

export interface LoginResponseData {
  user: UserProfile;
  backendToken: BackendToken;
}

export interface LoginResponse {
  success: boolean;
  data: LoginResponseData;
}