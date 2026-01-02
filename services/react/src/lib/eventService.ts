import { apiClient } from "./api";
import type { Event, EventPackage, CreateEventRequest, UpdateEventRequest, CreatePackageRequest, UpdatePackageRequest } from "./types";

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
    params.append("items_per_page", "100");

    const url = `${ENDPOINTS.EVENTS}?${params.toString()}`;

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
    params.append("items_per_page", filters?.itemsPerPage?.toString() ?? "100");

    const url = `${ENDPOINTS.EVENT_PACKETS}?${params.toString()}`;

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

  async createEvent(data: CreateEventRequest): Promise<Event> {
    const response = await this.eventService.post<Event>(ENDPOINTS.EVENTS, data);
    return response.data;
  }

  async updateEvent(id: number, data: UpdateEventRequest): Promise<Event> {
    const response = await this.eventService.put<Event>(`${ENDPOINTS.EVENTS}/${id}`, data);
    return response.data;
  }

  async deleteEvent(id: number): Promise<void> {
    await this.eventService.delete(`${ENDPOINTS.EVENTS}/${id}`);
  }

  async createPackage(data: CreatePackageRequest): Promise<EventPackage> {
    const response = await this.eventService.post<EventPackage>(ENDPOINTS.EVENT_PACKETS, data);
    return response.data;
  }

  async updatePackage(id: number, data: UpdatePackageRequest): Promise<EventPackage> {
    const response = await this.eventService.put<EventPackage>(`${ENDPOINTS.EVENT_PACKETS}/${id}`, data);
    return response.data;
  }

  async deletePackage(id: number): Promise<void> {
    await this.eventService.delete(`${ENDPOINTS.EVENT_PACKETS}/${id}`);
  }

  async addEventToPackage(packageId: number, eventId: number): Promise<void> {
    await this.eventService.post(`${ENDPOINTS.EVENT_PACKETS}/${packageId}/events/${eventId}`, {});
  }

  async removeEventFromPackage(packageId: number, eventId: number): Promise<void> {
    await this.eventService.delete(`${ENDPOINTS.EVENT_PACKETS}/${packageId}/events/${eventId}`);
  }

  async getTicketsForEvent(eventId: number): Promise<any[]> {
    const response = await this.eventService.get<any[]>(`${ENDPOINTS.EVENTS}/${eventId}/tickets`);
    return response.data;
  }

  async getTicketsForPackage(packageId: number): Promise<any[]> {
    const response = await this.eventService.get<any[]>(`${ENDPOINTS.EVENT_PACKETS}/${packageId}/tickets`);
    return response.data;
  }
}

export const eventService = new EventService();
