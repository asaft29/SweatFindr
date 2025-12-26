import { apiClient } from "./api";
import type { Event, EventPackage } from "./types";

const ENDPOINTS = {
  EVENTS: "/api/event-manager/events",
  EVENT_PACKETS: "/api/event-manager/event-packets",
} as const;

class EventService {
  private eventService = apiClient.getEventService();

  async getEvents(filters?: { locatie?: string; nume?: string }): Promise<Event[]> {
    const params = new URLSearchParams();
    if (filters?.locatie) params.append("location", filters.locatie);
    if (filters?.nume) params.append("name", filters.nume);

    const queryString = params.toString();
    const url = queryString ? `${ENDPOINTS.EVENTS}?${queryString}` : ENDPOINTS.EVENTS;

    const response = await this.eventService.get<Event[]>(url);
    return response.data;
  }

  async getEventById(id: number): Promise<Event> {
    const response = await this.eventService.get<Event>(`${ENDPOINTS.EVENTS}/${id}`);
    return response.data;
  }

  async getEventPackages(filters?: {
    type?: string;
    availableTickets?: number;
    page?: number;
    itemsPerPage?: number;
  }): Promise<EventPackage[]> {
    const params = new URLSearchParams();
    if (filters?.type) params.append("type", filters.type);
    if (filters?.availableTickets !== undefined) {
      params.append("available_tickets", filters.availableTickets.toString());
    }
    if (filters?.page !== undefined) params.append("page", filters.page.toString());
    if (filters?.itemsPerPage !== undefined) params.append("items_per_page", filters.itemsPerPage.toString());

    const queryString = params.toString();
    const url = queryString
      ? `${ENDPOINTS.EVENT_PACKETS}?${queryString}`
      : ENDPOINTS.EVENT_PACKETS;

    const response = await this.eventService.get<EventPackage[]>(url);
    return response.data;
  }

  async getEventPackageById(id: number): Promise<EventPackage> {
    const response = await this.eventService.get<EventPackage>(
      `${ENDPOINTS.EVENT_PACKETS}/${id}`
    );
    return response.data;
  }

  async getPackagesForEvent(eventId: number): Promise<EventPackage[]> {
    const response = await this.eventService.get<EventPackage[]>(
      `${ENDPOINTS.EVENTS}/${eventId}/event-packets`
    );
    return response.data;
  }

  async getEventsForPackage(packageId: number): Promise<Event[]> {
    const response = await this.eventService.get<Event[]>(
      `${ENDPOINTS.EVENT_PACKETS}/${packageId}/events`
    );
    return response.data;
  }
}

export const eventService = new EventService();
