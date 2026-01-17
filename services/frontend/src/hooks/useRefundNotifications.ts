import { useCallback } from 'react';
import { useWebSocket } from './useWebSocket';
import type { WebSocketMessage } from '../lib/websocketService';

interface RefundNotificationCallbacks {
  onRefundStatusChanged?: (data: {
    request_id: number;
    ticket_cod: string;
    status: string;
    event_name?: string;
    message?: string;
    user_id: number;
  }) => void;
  onNewRefundRequest?: (data: {
    request_id: number;
    ticket_cod: string;
    requester_email: string;
    event_id?: number;
    packet_id?: number;
    reason: string;
    created_at: string;
    event_owner_id: number;
  }) => void;
}

export function useRefundNotifications(callbacks: RefundNotificationCallbacks) {
  const handler = useCallback((message: WebSocketMessage) => {
    switch (message.type) {
      case 'refund_status_changed':
        callbacks.onRefundStatusChanged?.(message);
        break;
      case 'new_refund_request':
        callbacks.onNewRefundRequest?.(message);
        break;
    }
  }, [callbacks]);

  return useWebSocket(handler);
}
