export const config = {
  api: {
    gateway: import.meta.env.VITE_API_GATEWAY_URL || "http://localhost:10000",
    clientService:
      import.meta.env.VITE_API_CLIENT_SERVICE_URL || "http://localhost:8002",
    eventService:
      import.meta.env.VITE_API_EVENT_SERVICE_URL || "http://localhost:8001",
  },
} as const;
