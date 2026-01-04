import axios, { AxiosInstance, AxiosRequestConfig, AxiosError } from "axios";
import { config } from "./config";

class ApiClient {
  private gateway: AxiosInstance;
  private clientService: AxiosInstance;
  private eventService: AxiosInstance;

  constructor() {
    this.gateway = axios.create({
      baseURL: config.api.gateway,
      headers: {
        "Content-Type": "application/json",
      },
    });

    this.clientService = axios.create({
      baseURL: config.api.clientService,
      headers: {
        "Content-Type": "application/json",
      },
    });

    this.eventService = axios.create({
      baseURL: config.api.eventService,
      headers: {
        "Content-Type": "application/json",
      },
    });

    this.setupInterceptors();
  }

  private setupInterceptors() {
    const requestInterceptor = (config: AxiosRequestConfig) => {
      const token = localStorage.getItem("auth_token");
      if (token && config.headers) {
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    };

    this.gateway.interceptors.request.use(requestInterceptor as any);
    this.clientService.interceptors.request.use(requestInterceptor as any);
    this.eventService.interceptors.request.use(requestInterceptor as any);

    const responseErrorInterceptor = (error: AxiosError) => {
      if (error.response?.status === 401) {
        localStorage.removeItem("auth_token");
        localStorage.removeItem("user");
        window.location.href = "/login";
      }
      if (error.response?.status === 429) {
        const headers = error.response.headers;
        const retryAfter =
          headers["retry-after"] ||
          headers["x-ratelimit-after"] ||
          headers["Retry-After"] ||
          headers["X-Ratelimit-After"];
        const waitTime = retryAfter ? parseInt(String(retryAfter), 10) : 60;
        const rateLimitError = new Error(
          `Too many requests. Please try again in ${waitTime} seconds.`
        ) as AxiosError;
        rateLimitError.response = error.response;
        rateLimitError.name = "RateLimitError";
        return Promise.reject(rateLimitError);
      }
      return Promise.reject(error);
    };

    this.gateway.interceptors.response.use(
      (response) => response,
      responseErrorInterceptor
    );
    this.clientService.interceptors.response.use(
      (response) => response,
      responseErrorInterceptor
    );
    this.eventService.interceptors.response.use(
      (response) => response,
      responseErrorInterceptor
    );
  }

  getGateway() {
    return this.gateway;
  }

  getClientService() {
    return this.clientService;
  }

  getEventService() {
    return this.eventService;
  }
}

export const apiClient = new ApiClient();
