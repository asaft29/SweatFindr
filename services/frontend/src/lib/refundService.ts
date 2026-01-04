import { apiClient } from "./api";
import { RefundRequest, CreateRefundRequest, RefundResponse } from "./types";

const eventService = apiClient.getEventService();
const clientServiceApi = apiClient.getClientService();

export const getOwnerRefundRequests = async (): Promise<RefundRequest[]> => {
  const response = await eventService.get<{ data: RefundRequest[] }>(
    "/api/event-manager/refunds"
  );
  return response.data?.data || [];
};

export const approveRefund = async (refundId: number): Promise<void> => {
  await eventService.post(`/api/event-manager/refunds/${refundId}/approve`);
};

export const rejectRefund = async (
  refundId: number,
  reason: string
): Promise<void> => {
  await eventService.post(`/api/event-manager/refunds/${refundId}/reject`, {
    message: reason,
  });
};

export const getClientRefunds = async (
  email: string
): Promise<RefundRequest[]> => {
  const response = await eventService.get<{ data: RefundRequest[] }>(
    `/api/event-manager/refunds/history?email=${encodeURIComponent(email)}`
  );
  return response.data?.data || [];
};

export const submitRefundRequest = async (
  clientId: string,
  request: CreateRefundRequest
): Promise<RefundResponse> => {
  const response = await clientServiceApi.post<RefundResponse>(
    `/api/client-manager/clients/${clientId}/refunds`,
    request
  );
  return response.data;
};
