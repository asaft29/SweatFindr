import { useEffect } from "react";
import {
  websocketService,
  useWebSocketStore,
  type WebSocketMessage,
} from "../lib/websocketService";
import { useAuthStore } from "../lib/useAuthStore";

export function useWebSocket(handler: (message: WebSocketMessage) => void) {
  const token = useAuthStore((state) => state.token);
  const connected = useWebSocketStore((state) => state.connected);

  useEffect(() => {
    if (!token) {
      return;
    }

    websocketService.connect(token);

    const unsubscribe = websocketService.subscribe(handler);

    return () => {
      unsubscribe();
    };
  }, [token, handler]);

  useEffect(() => {
    return () => {
      websocketService.disconnect();
    };
  }, []);

  return { connected };
}
