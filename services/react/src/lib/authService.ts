import { apiClient } from "./api";
import type {
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  RegisterResponse,
  LogoutRequest,
  LogoutResponse,
  VerifyEmailRequest,
  VerifyEmailResponse,
  ResendVerificationRequest,
  ResendVerificationResponse,
} from "../lib/types";

class AuthService {
  private gateway = apiClient.getGateway();

  async login(credentials: LoginRequest): Promise<LoginResponse> {
    const response = await this.gateway.post<LoginResponse>(
      "/api/gateway/auth/login",
      credentials
    );
    return response.data;
  }

  async register(data: RegisterRequest): Promise<RegisterResponse> {
    const response = await this.gateway.post<RegisterResponse>(
      "/api/gateway/auth/register",
      data
    );
    return response.data;
  }

  async logout(token: string): Promise<LogoutResponse> {
    const response = await this.gateway.post<LogoutResponse>(
      "/api/gateway/auth/logout",
      { token_value: token } as LogoutRequest
    );
    return response.data;
  }

  async verifyEmail(data: VerifyEmailRequest): Promise<VerifyEmailResponse> {
    const response = await this.gateway.post<VerifyEmailResponse>(
      "/api/gateway/email/verify",
      data
    );
    return response.data;
  }

  async resendVerification(
    data: ResendVerificationRequest
  ): Promise<ResendVerificationResponse> {
    const response = await this.gateway.post<ResendVerificationResponse>(
      "/api/gateway/email/resend",
      data
    );
    return response.data;
  }
}

export const authService = new AuthService();
