import { create } from 'zustand';

export type WebSocketMessage =
  | { type: 'refund_status_changed'; request_id: number; ticket_cod: string; status: string; event_name?: string; message?: string; user_id: number }
  | { type: 'new_refund_request'; request_id: number; ticket_cod: string; requester_email: string; event_id?: number; packet_id?: number; reason: string; created_at: string; event_owner_id: number };

type MessageHandler = (message: WebSocketMessage) => void;

interface WebSocketState {
  connected: boolean;
  setConnected: (connected: boolean) => void;
}

export const useWebSocketStore = create<WebSocketState>((set) => ({
  connected: false,
  setConnected: (connected: boolean) => set({ connected }),
}));

class WebSocketService {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectDelay = 30000;
  private baseDelay = 1000;
  private handlers: Set<MessageHandler> = new Set();
  private pingInterval: ReturnType<typeof setInterval> | null = null;
  private shouldReconnect = false;
  private token: string | null = null;

  connect(token: string) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    this.token = token;
    this.shouldReconnect = true;

    const wsUrl = `ws://localhost:8004/ws?token=${token}`;

    try {
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        useWebSocketStore.getState().setConnected(true);
        this.reconnectAttempts = 0;
        this.startPing();
      };

      this.ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          this.handlers.forEach(handler => handler(message));
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      this.ws.onclose = () => {
        useWebSocketStore.getState().setConnected(false);
        this.stopPing();

        if (this.shouldReconnect && this.token) {
          this.reconnect(this.token);
        }
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      useWebSocketStore.getState().setConnected(false);
    }
  }

  private reconnect(token: string) {
    if (!this.shouldReconnect) {
      return;
    }

    const delay = Math.min(
      this.baseDelay * Math.pow(2, this.reconnectAttempts) + Math.random() * 1000,
      this.maxReconnectDelay
    );
    this.reconnectAttempts++;

    setTimeout(() => this.connect(token), delay);
  }

  private startPing() {
    this.pingInterval = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send('ping');
      }
    }, 30000);
  }

  private stopPing() {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }

  subscribe(handler: MessageHandler) {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  disconnect() {
    this.shouldReconnect = false;
    this.stopPing();
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    useWebSocketStore.getState().setConnected(false);
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}

export const websocketService = new WebSocketService();
