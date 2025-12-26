export enum UserRole {
  GUEST = "guest",
  CLIENT = "client",
  OWNER_EVENT = "owner-event",
}

export interface User {
  id: number;
  email: string;
  role: UserRole;
  emailVerified: boolean;
}

export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  success: boolean;
  message: string;
  token_value?: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  role: string;
}

export interface RegisterResponse {
  success: boolean;
  message: string;
  user_id?: number;
  token_value?: string;
}

export interface LogoutRequest {
  token_value: string;
}

export interface LogoutResponse {
  success: boolean;
  message: string;
}

export interface VerifyEmailRequest {
  user_id: number;
  verification_code: string;
}

export interface VerifyEmailResponse {
  success: boolean;
  message: string;
}

export interface ResendVerificationRequest {
  user_id: number;
  email: string;
}

export interface ResendVerificationResponse {
  success: boolean;
  message: string;
}

export interface Event {
  id: number;
  id_owner: number;
  nume: string;
  locatie: string | null;
  descriere: string | null;
  numarlocuri: number | null;
}

export interface EventPackage {
  id: number;
  id_owner: number;
  nume: string;
  locatie: string | null;
  descriere: string | null;
  numarlocuri: number | null;
}
