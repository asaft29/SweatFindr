import { apiClient } from "./api";
import type { Client, TicketRef, TicketBuyerInfo } from "./types";

interface PurchaseTicketRequest {
  evenimentid?: number;
  pachetid?: number;
}

class ClientService {
  private clientService = apiClient.getClientService();
  private cachedClientId: string | null = null;

  private async getMyClientId(): Promise<string> {
    if (this.cachedClientId) {
      return this.cachedClientId;
    }
    const response = await this.clientService.get<any>(
      `/api/client-manager/clients/me`
    );
    if (!response.data?._id) {
      throw new Error("Client profile not found");
    }
    const id =
      typeof response.data._id === "object" && response.data._id.$oid
        ? response.data._id.$oid
        : String(response.data._id);
    this.cachedClientId = id;
    return id;
  }

  clearCache(): void {
    this.cachedClientId = null;
  }

  async getMyProfile(): Promise<Client> {
    const response = await this.clientService.get<any>(
      `/api/client-manager/clients/me`
    );
    const data = response.data;
    if (data._id && typeof data._id === "object" && data._id.$oid) {
      data._id = data._id.$oid;
    }
    if (data._id) {
      this.cachedClientId = data._id;
    }
    return data as Client;
  }

  async updateMyProfile(data: Partial<Client>): Promise<Client> {
    const clientId = await this.getMyClientId();
    await this.clientService.put(
      `/api/client-manager/clients/${clientId}`,
      data
    );
    return this.getMyProfile();
  }

  async getMyTickets(): Promise<TicketRef[]> {
    const clientId = await this.getMyClientId();
    const response = await this.clientService.get<TicketRef[]>(
      `/api/client-manager/clients/${clientId}/tickets`
    );
    return response.data;
  }

  async purchaseTicket(request: PurchaseTicketRequest): Promise<Client> {
    const clientId = await this.getMyClientId();
    const response = await this.clientService.post<Client>(
      `/api/client-manager/clients/${clientId}/tickets`,
      request
    );
    return response.data;
  }

  async getBuyerByTicketCode(ticketCode: string): Promise<TicketBuyerInfo> {
    const response = await this.clientService.get<TicketBuyerInfo>(
      `/api/client-manager/clients/data/${ticketCode}`
    );
    return response.data;
  }

  async getClientByEmail(email: string): Promise<Client | null> {
    try {
      const response = await this.clientService.get<Client[]>(
        `/api/client-manager/clients?email=${encodeURIComponent(email)}`
      );
      const clients = response.data;
      return clients && clients.length > 0 ? clients[0] : null;
    } catch {
      return null;
    }
  }

  async deleteMyAccount(): Promise<void> {
    await this.clientService.delete(`/api/client-manager/clients/me`);
    this.clearCache();
  }

  async requestRefund(
    ticketCode: string,
    reason: string
  ): Promise<{ message: string }> {
    const clientId = await this.getMyClientId();
    const response = await this.clientService.post<{ message: string }>(
      `/api/client-manager/clients/${clientId}/refunds`,
      { ticket_cod: ticketCode, reason }
    );
    return response.data;
  }
}

export const clientService = new ClientService();

export const getClientByEmail = (email: string) =>
  clientService.getClientByEmail(email);
