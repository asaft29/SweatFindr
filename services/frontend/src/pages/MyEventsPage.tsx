import { useEffect, useState, useRef, useCallback } from "react";
import { eventService } from "../lib/eventService";
import { clientService } from "../lib/clientService";
import { useAuthStore } from "../lib/useAuthStore";
import { ConfirmModal } from "../components/ConfirmModal";
import { ErrorModal } from "../components/ErrorModal";
import type { Event, EventPackage, CreateEventRequest, CreatePackageRequest, TicketBuyerInfo } from "../lib/types";

interface Ticket {
  cod: string;
  pachetid: number | null;
  evenimentid: number | null;
}

interface TicketWithBuyer extends Ticket {
  buyer?: TicketBuyerInfo;
  buyerLoading?: boolean;
}

interface ValidationErrors {
  nume?: string;
  locatie?: string;
  descriere?: string;
  numarlocuri?: string;
}

interface LocationSuggestion {
  display_name: string;
  place_id: number;
}

type TabType = "events" | "packages";

const validateEvent = (form: CreateEventRequest): ValidationErrors => {
  const errors: ValidationErrors = {};

  if (form.nume.length < 3 || form.nume.length > 100) {
    errors.nume = "Name must be between 3 and 100 characters";
  }

  if (form.locatie && form.locatie.length > 255) {
    errors.locatie = "Location must be less than 255 characters";
  }

  if (form.descriere && (form.descriere.length < 10 || form.descriere.length > 500)) {
    errors.descriere = "Description must be between 10 and 500 characters";
  }

  if (form.numarlocuri !== undefined && form.numarlocuri !== 0) {
    if (form.numarlocuri < 1 || form.numarlocuri > 50000) {
      errors.numarlocuri = "Seats must be between 1 and 50,000";
    }
  }

  return errors;
};

const validatePackage = (form: CreatePackageRequest): ValidationErrors => {
  const errors: ValidationErrors = {};

  if (form.nume.length < 3 || form.nume.length > 100) {
    errors.nume = "Name must be between 3 and 100 characters";
  }

  if (form.locatie && form.locatie.length > 255) {
    errors.locatie = "Location must be less than 255 characters";
  }

  if (form.descriere && (form.descriere.length < 10 || form.descriere.length > 500)) {
    errors.descriere = "Description must be between 10 and 500 characters";
  }

  return errors;
};

const isFormValid = (errors: ValidationErrors, form: CreateEventRequest | CreatePackageRequest): boolean => {
  return Object.keys(errors).length === 0 && form.nume.length >= 3;
};

export function MyEventsPage() {
  const { user } = useAuthStore();
  const [activeTab, setActiveTab] = useState<TabType>("events");
  const [events, setEvents] = useState<Event[]>([]);
  const [packages, setPackages] = useState<EventPackage[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorModal, setErrorModal] = useState<{ title: string; message: string } | null>(null);

  const [showEventModal, setShowEventModal] = useState(false);
  const [showPackageModal, setShowPackageModal] = useState(false);
  const [showTicketsModal, setShowTicketsModal] = useState(false);
  const [showBindEventsModal, setShowBindEventsModal] = useState(false);
  const [editingEvent, setEditingEvent] = useState<Event | null>(null);
  const [editingPackage, setEditingPackage] = useState<EventPackage | null>(null);
  const [viewingTickets, setViewingTickets] = useState<{ type: "event" | "package"; id: number; name: string } | null>(null);
  const [tickets, setTickets] = useState<TicketWithBuyer[]>([]);
  const [ticketsLoading, setTicketsLoading] = useState(false);
  const [bindingPackage, setBindingPackage] = useState<EventPackage | null>(null);
  const [packageEvents, setPackageEvents] = useState<Event[]>([]);
  const [bindingLoading, setBindingLoading] = useState(false);

  const [deleteConfirm, setDeleteConfirm] = useState<{ type: "event" | "package"; id: number; name: string } | null>(null);

  const [eventForm, setEventForm] = useState<CreateEventRequest>({
    nume: "",
    locatie: "",
    descriere: "",
    numarlocuri: 0,
  });
  const [eventErrors, setEventErrors] = useState<ValidationErrors>({});

  const [packageForm, setPackageForm] = useState<CreatePackageRequest>({
    nume: "",
    locatie: "",
    descriere: "",
  });
  const [packageErrors, setPackageErrors] = useState<ValidationErrors>({});

  const [locationSuggestions, setLocationSuggestions] = useState<LocationSuggestion[]>([]);
  const [showLocationSuggestions, setShowLocationSuggestions] = useState(false);
  const [locationInputFocused, setLocationInputFocused] = useState<"event" | "package" | null>(null);
  const [locationLoading, setLocationLoading] = useState(false);
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const searchLocations = useCallback(async (query: string) => {
    if (query.length < 2) {
      setLocationSuggestions([]);
      return;
    }

    setLocationLoading(true);
    try {
      const response = await fetch(
        `https://nominatim.openstreetmap.org/search?format=json&q=${encodeURIComponent(query)}&limit=5&addressdetails=1`,
        {
          headers: {
            'Accept-Language': 'en',
          },
        }
      );
      const data = await response.json();
      setLocationSuggestions(data.map((item: any) => ({
        display_name: item.display_name,
        place_id: item.place_id,
      })));
    } catch (err) {
      console.error("Failed to fetch location suggestions:", err);
      setLocationSuggestions([]);
    } finally {
      setLocationLoading(false);
    }
  }, []);

  const handleLocationChange = useCallback((value: string, target: "event" | "package") => {
    if (target === "event") {
      setEventForm(prev => ({ ...prev, locatie: value }));
    } else {
      setPackageForm(prev => ({ ...prev, locatie: value }));
    }

    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    debounceTimerRef.current = setTimeout(() => {
      searchLocations(value);
    }, 300);
  }, [searchLocations]);

  useEffect(() => {
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    loadData();
  }, []);

  useEffect(() => {
    setEventErrors(validateEvent(eventForm));
  }, [eventForm]);

  useEffect(() => {
    setPackageErrors(validatePackage(packageForm));
  }, [packageForm]);

  const loadData = async () => {
    try {
      setLoading(true);
      const [eventsData, packagesData] = await Promise.all([
        eventService.getEvents(),
        eventService.getEventPackages(),
      ]);
      const userId = Number(user?.id);
      const myEvents = eventsData.filter((e) => Number(e.id_owner) === userId);
      const myPackages = packagesData.filter((p) => Number(p.id_owner) === userId);
      setEvents(myEvents);
      setPackages(myPackages);
    } catch (err: any) {
      setErrorModal({
        title: "Error Loading Data",
        message: err.response?.data?.error || err.message || "Failed to load data"
      });
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateEvent = async () => {
    const errors = validateEvent(eventForm);
    if (!isFormValid(errors, eventForm)) {
      setEventErrors(errors);
      return;
    }
    try {
      const payload = { ...eventForm };
      if (payload.numarlocuri === 0) payload.numarlocuri = undefined;
      if (!payload.locatie) payload.locatie = undefined;
      if (!payload.descriere) payload.descriere = undefined;
      await eventService.createEvent(payload);
      setShowEventModal(false);
      setEventForm({ nume: "", locatie: "", descriere: "", numarlocuri: 0 });
      await loadData();
    } catch (err: any) {
      const details = err.response?.data?.details;
      const message = Array.isArray(details) && details.length > 0
        ? details[0]
        : err.response?.data?.error || err.message || "Failed to create event";
      setErrorModal({
        title: "Failed to Create Event",
        message
      });
    }
  };

  const handleUpdateEvent = async () => {
    if (!editingEvent) return;
    const errors = validateEvent(eventForm);
    if (!isFormValid(errors, eventForm)) {
      setEventErrors(errors);
      return;
    }
    try {
      const payload = { ...eventForm };
      if (payload.numarlocuri === 0) payload.numarlocuri = undefined;
      if (!payload.locatie) payload.locatie = undefined;
      if (!payload.descriere) payload.descriere = undefined;
      await eventService.updateEvent(editingEvent.id, payload);
      setShowEventModal(false);
      setEditingEvent(null);
      setEventForm({ nume: "", locatie: "", descriere: "", numarlocuri: 0 });
      await loadData();
    } catch (err: any) {
      const details = err.response?.data?.details;
      const message = Array.isArray(details) && details.length > 0
        ? details[0]
        : err.response?.data?.error || err.message || "Failed to update event";
      setErrorModal({
        title: "Failed to Update Event",
        message
      });
    }
  };

  const handleDeleteEvent = async (id: number) => {
    try {
      await eventService.deleteEvent(id);
      await loadData();
    } catch (err: any) {
      const details = err.response?.data?.details;
      const message = Array.isArray(details) && details.length > 0
        ? details[0]
        : err.response?.data?.error || err.message || "Failed to delete event";
      setErrorModal({
        title: "Failed to Delete Event",
        message
      });
    }
  };

  const confirmDelete = async () => {
    if (!deleteConfirm) return;
    setDeleteConfirm(null);
    if (deleteConfirm.type === "event") {
      await handleDeleteEvent(deleteConfirm.id);
    } else {
      await handleDeletePackage(deleteConfirm.id);
    }
  };

  const handleCreatePackage = async () => {
    const errors = validatePackage(packageForm);
    if (!isFormValid(errors, packageForm)) {
      setPackageErrors(errors);
      return;
    }
    try {
      const payload = { ...packageForm };
      if (!payload.locatie) payload.locatie = undefined;
      if (!payload.descriere) payload.descriere = undefined;
      await eventService.createPackage(payload);
      setShowPackageModal(false);
      setPackageForm({ nume: "", locatie: "", descriere: "" });
      await loadData();
    } catch (err: any) {
      const details = err.response?.data?.details;
      const message = Array.isArray(details) && details.length > 0
        ? details[0]
        : err.response?.data?.error || err.message || "Failed to create package";
      setErrorModal({
        title: "Failed to Create Package",
        message
      });
    }
  };

  const handleUpdatePackage = async () => {
    if (!editingPackage) return;
    const errors = validatePackage(packageForm);
    if (!isFormValid(errors, packageForm)) {
      setPackageErrors(errors);
      return;
    }
    try {
      const payload = { ...packageForm };
      if (!payload.locatie) payload.locatie = undefined;
      if (!payload.descriere) payload.descriere = undefined;
      await eventService.updatePackage(editingPackage.id, payload);
      setShowPackageModal(false);
      setEditingPackage(null);
      setPackageForm({ nume: "", locatie: "", descriere: "" });
      await loadData();
    } catch (err: any) {
      const details = err.response?.data?.details;
      const message = Array.isArray(details) && details.length > 0
        ? details[0]
        : err.response?.data?.error || err.message || "Failed to update package";
      setErrorModal({
        title: "Failed to Update Package",
        message
      });
    }
  };

  const handleDeletePackage = async (id: number) => {
    try {
      await eventService.deletePackage(id);
      await loadData();
    } catch (err: any) {
      const details = err.response?.data?.details;
      const message = Array.isArray(details) && details.length > 0
        ? details[0]
        : err.response?.data?.error || err.message || "Failed to delete package";
      setErrorModal({
        title: "Failed to Delete Package",
        message
      });
    }
  };

  const openEditEvent = (event: Event) => {
    setEditingEvent(event);
    setEventForm({
      nume: event.nume,
      locatie: event.locatie || "",
      descriere: event.descriere || "",
      numarlocuri: event.numarlocuri || 0,
    });
    setShowEventModal(true);
  };

  const openEditPackage = (pkg: EventPackage) => {
    setEditingPackage(pkg);
    setPackageForm({
      nume: pkg.nume,
      locatie: pkg.locatie || "",
      descriere: pkg.descriere || "",
    });
    setShowPackageModal(true);
  };

  const openCreateEvent = () => {
    setEditingEvent(null);
    setEventForm({ nume: "", locatie: "", descriere: "", numarlocuri: 0 });
    setEventErrors({});
    setLocationSuggestions([]);
    setShowEventModal(true);
  };

  const openCreatePackage = () => {
    setEditingPackage(null);
    setPackageForm({ nume: "", locatie: "", descriere: "" });
    setPackageErrors({});
    setLocationSuggestions([]);
    setShowPackageModal(true);
  };

  const openViewTickets = async (type: "event" | "package", id: number, name: string) => {
    setViewingTickets({ type, id, name });
    setTicketsLoading(true);
    setShowTicketsModal(true);
    try {
      const ticketsData: Ticket[] = type === "event"
        ? await eventService.getTicketsForEvent(id)
        : await eventService.getTicketsForPackage(id);

      const ticketsWithBuyers: TicketWithBuyer[] = ticketsData.map(t => ({ ...t, buyerLoading: true }));
      setTickets(ticketsWithBuyers);

      for (const ticket of ticketsWithBuyers) {
        try {
          const buyer = await clientService.getBuyerByTicketCode(ticket.cod);
          setTickets(prev => prev.map(t =>
            t.cod === ticket.cod ? { ...t, buyer, buyerLoading: false } : t
          ));
        } catch {
          setTickets(prev => prev.map(t =>
            t.cod === ticket.cod ? { ...t, buyerLoading: false } : t
          ));
        }
      }
    } catch (err: any) {
      setErrorModal({
        title: "Failed to Load Tickets",
        message: err.response?.data?.error || err.message || "Failed to load tickets"
      });
      setTickets([]);
    } finally {
      setTicketsLoading(false);
    }
  };

  const openBindEvents = async (pkg: EventPackage) => {
    setBindingPackage(pkg);
    setBindingLoading(true);
    setShowBindEventsModal(true);
    try {
      const boundEvents = await eventService.getEventsForPackage(pkg.id);
      setPackageEvents(boundEvents);
    } catch (err: any) {
      setErrorModal({
        title: "Failed to Load Package Events",
        message: err.response?.data?.error || err.message || "Failed to load package events"
      });
      setPackageEvents([]);
    } finally {
      setBindingLoading(false);
    }
  };

  const handleBindEvent = async (eventId: number) => {
    if (!bindingPackage) return;
    try {
      setBindingLoading(true);
      await eventService.addEventToPackage(bindingPackage.id, eventId);
      const boundEvents = await eventService.getEventsForPackage(bindingPackage.id);
      setPackageEvents(boundEvents);
      await loadData();
    } catch (err: any) {
      const details = err.response?.data?.details;
      const message = Array.isArray(details) && details.length > 0
        ? details[0]
        : err.response?.data?.error || err.message || "Failed to bind event";
      setErrorModal({
        title: "Failed to Add Event",
        message
      });
    } finally {
      setBindingLoading(false);
    }
  };

  const handleUnbindEvent = async (eventId: number) => {
    if (!bindingPackage) return;
    try {
      setBindingLoading(true);
      await eventService.removeEventFromPackage(bindingPackage.id, eventId);
      const boundEvents = await eventService.getEventsForPackage(bindingPackage.id);
      setPackageEvents(boundEvents);
      await loadData();
    } catch (err: any) {
      const details = err.response?.data?.details;
      const message = Array.isArray(details) && details.length > 0
        ? details[0]
        : err.response?.data?.error || err.message || "Failed to unbind event";
      setErrorModal({
        title: "Failed to Remove Event",
        message
      });
    } finally {
      setBindingLoading(false);
    }
  };

  const isEventBound = (eventId: number) => {
    return packageEvents.some(e => e.id === eventId);
  };

  const selectLocation = (location: string, target: "event" | "package") => {
    if (target === "event") {
      setEventForm({ ...eventForm, locatie: location });
    } else {
      setPackageForm({ ...packageForm, locatie: location });
    }
    setShowLocationSuggestions(false);
    setLocationSuggestions([]);
  };

  const renderValidationMessage = (error?: string, currentLength?: number, maxLength?: number) => {
    if (error) {
      return <p className="text-red-500 text-xs mt-1">{error}</p>;
    }
    if (currentLength !== undefined && maxLength !== undefined) {
      return <p className="text-gray-400 text-xs mt-1">{currentLength}/{maxLength}</p>;
    }
    return null;
  };

  const renderLocationInput = (target: "event" | "package") => {
    const value = target === "event" ? eventForm.locatie : packageForm.locatie;
    const errors = target === "event" ? eventErrors : packageErrors;

    return (
      <div className="relative">
        <label className="block text-sm font-medium text-gray-700 mb-1">Location</label>
        <div className="relative">
          <input
            type="text"
            value={value}
            onChange={(e) => handleLocationChange(e.target.value, target)}
            onFocus={() => {
              setLocationInputFocused(target);
              setShowLocationSuggestions(true);
              if (value && value.length >= 2) {
                searchLocations(value);
              }
            }}
            onBlur={() => setTimeout(() => setShowLocationSuggestions(false), 200)}
            maxLength={255}
            placeholder="Start typing to search locations..."
            className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 ${errors.locatie ? "border-red-500" : "border-gray-300"
              }`}
          />
          {locationLoading && locationInputFocused === target && (
            <div className="absolute right-3 top-1/2 -translate-y-1/2">
              <div className="animate-spin rounded-full h-4 w-4 border-2 border-indigo-600 border-t-transparent"></div>
            </div>
          )}
        </div>
        {showLocationSuggestions && locationInputFocused === target && locationSuggestions.length > 0 && (
          <div className="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-lg shadow-lg max-h-48 overflow-y-auto">
            {locationSuggestions.map((suggestion) => (
              <button
                key={suggestion.place_id}
                type="button"
                onClick={() => selectLocation(suggestion.display_name, target)}
                className="w-full px-4 py-2 text-left text-sm hover:bg-indigo-50 focus:bg-indigo-50 border-b border-gray-100 last:border-b-0"
              >
                <span className="line-clamp-2">{suggestion.display_name}</span>
              </button>
            ))}
          </div>
        )}
        {renderValidationMessage(errors.locatie, value?.length, 255)}
      </div>
    );
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex gap-4 mb-8">
          <button
            onClick={() => setActiveTab("events")}
            className={`px-6 py-2.5 text-sm font-bold rounded-lg transition ${activeTab === "events"
              ? "bg-indigo-600 text-white"
              : "bg-white text-indigo-600 hover:bg-indigo-50"
              }`}
          >
            Events
          </button>
          <button
            onClick={() => setActiveTab("packages")}
            className={`px-6 py-2.5 text-sm font-bold rounded-lg transition ${activeTab === "packages"
              ? "bg-indigo-600 text-white"
              : "bg-white text-indigo-600 hover:bg-indigo-50"
              }`}
          >
            Packages
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center py-12">
            <div className="inline-block animate-spin rounded-full h-12 w-12 border-t-4 border-b-4 border-indigo-600"></div>
          </div>
        ) : activeTab === "events" ? (
          <div>
            <div className="flex justify-end mb-6">
              <button
                onClick={openCreateEvent}
                className="px-6 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition shadow-md"
              >
                + Create Event
              </button>
            </div>

            {events.length === 0 ? (
              <div className="text-center py-12">
                <p className="text-xl text-gray-600">You haven't created any events yet.</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {events.map((event) => (
                  <div key={event.id} className="bg-white rounded-xl shadow-lg p-6 hover:shadow-2xl transition-shadow duration-300 flex flex-col h-full">
                    <h3 className="text-2xl font-bold text-gray-900 mb-3">{event.nume}</h3>
                    <p className="text-indigo-600 font-medium mb-2">{event.locatie || "Location not specified"}</p>
                    <p className="text-gray-700 mb-4 flex-grow">{event.descriere || "No description available"}</p>
                    <p className="text-sm text-gray-500 font-medium mb-4">
                      Available seats: {event.numarlocuri ?? 0}
                    </p>
                    <div className="flex flex-col gap-2 mt-auto">
                      <button
                        onClick={() => openViewTickets("event", event.id, event.nume)}
                        className="w-full px-4 py-2 text-sm font-bold text-green-600 bg-green-50 hover:bg-green-100 rounded-lg transition"
                      >
                        View Tickets
                      </button>
                      <div className="flex gap-2">
                        <button
                          onClick={() => openEditEvent(event)}
                          className="flex-1 px-4 py-2 text-sm font-bold text-indigo-600 bg-indigo-50 hover:bg-indigo-100 rounded-lg transition"
                        >
                          Edit
                        </button>
                        <button
                          onClick={() => setDeleteConfirm({ type: "event", id: event.id, name: event.nume })}
                          className="flex-1 px-4 py-2 text-sm font-bold text-red-600 bg-red-50 hover:bg-red-100 rounded-lg transition"
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        ) : (
          <div>
            <div className="flex justify-end mb-6">
              <button
                onClick={openCreatePackage}
                className="px-6 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition shadow-md"
              >
                + Create Package
              </button>
            </div>

            {packages.length === 0 ? (
              <div className="text-center py-12">
                <p className="text-xl text-gray-600">You haven't created any packages yet.</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {packages.map((pkg) => (
                  <div key={pkg.id} className="bg-white rounded-xl shadow-lg p-6 hover:shadow-2xl transition-shadow duration-300 flex flex-col h-full">
                    <h3 className="text-2xl font-bold text-gray-900 mb-3">{pkg.nume}</h3>
                    <p className="text-indigo-600 font-medium mb-2">{pkg.locatie || "Location not specified"}</p>
                    <p className="text-gray-700 mb-4 flex-grow">{pkg.descriere || "No description available"}</p>
                    <p className="text-sm text-gray-500 font-medium mb-4">
                      Available seats: {pkg.numarlocuri ?? 0}
                    </p>
                    <div className="flex flex-col gap-2 mt-auto">
                      <button
                        onClick={() => openBindEvents(pkg)}
                        className="w-full px-4 py-2 text-sm font-bold text-purple-600 bg-purple-50 hover:bg-purple-100 rounded-lg transition"
                      >
                        Manage Events
                      </button>
                      <button
                        onClick={() => openViewTickets("package", pkg.id, pkg.nume)}
                        className="w-full px-4 py-2 text-sm font-bold text-green-600 bg-green-50 hover:bg-green-100 rounded-lg transition"
                      >
                        View Tickets
                      </button>
                      <div className="flex gap-2">
                        <button
                          onClick={() => openEditPackage(pkg)}
                          className="flex-1 px-4 py-2 text-sm font-bold text-indigo-600 bg-indigo-50 hover:bg-indigo-100 rounded-lg transition"
                        >
                          Edit
                        </button>
                        <button
                          onClick={() => setDeleteConfirm({ type: "package", id: pkg.id, name: pkg.nume })}
                          className="flex-1 px-4 py-2 text-sm font-bold text-red-600 bg-red-50 hover:bg-red-100 rounded-lg transition"
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {showEventModal && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-white rounded-xl shadow-2xl p-8 w-full max-w-md max-h-[90vh] overflow-y-auto">
              <h2 className="text-2xl font-bold text-gray-900 mb-6">
                {editingEvent ? "Edit Event" : "Create Event"}
              </h2>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Name <span className="text-red-500">*</span>
                  </label>
                  <input
                    type="text"
                    value={eventForm.nume}
                    onChange={(e) => setEventForm({ ...eventForm, nume: e.target.value })}
                    maxLength={100}
                    className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 ${eventErrors.nume ? "border-red-500" : "border-gray-300"
                      }`}
                  />
                  {renderValidationMessage(eventErrors.nume, eventForm.nume.length, 100)}
                </div>
                {renderLocationInput("event")}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
                  <textarea
                    value={eventForm.descriere}
                    onChange={(e) => setEventForm({ ...eventForm, descriere: e.target.value })}
                    rows={3}
                    maxLength={500}
                    placeholder="Min 10 characters if provided"
                    className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 ${eventErrors.descriere ? "border-red-500" : "border-gray-300"
                      }`}
                  />
                  {renderValidationMessage(eventErrors.descriere, eventForm.descriere?.length, 500)}
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Available Seats</label>
                  <input
                    type="number"
                    value={eventForm.numarlocuri || ""}
                    onChange={(e) => setEventForm({ ...eventForm, numarlocuri: parseInt(e.target.value) || 0 })}
                    min={0}
                    max={50000}
                    placeholder="1 - 50,000"
                    className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 ${eventErrors.numarlocuri ? "border-red-500" : "border-gray-300"
                      }`}
                  />
                  {renderValidationMessage(eventErrors.numarlocuri)}
                </div>
              </div>
              <div className="flex gap-3 mt-6">
                <button
                  onClick={() => {
                    setShowEventModal(false);
                    setEditingEvent(null);
                    setLocationSuggestions([]);
                  }}
                  className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition"
                >
                  Cancel
                </button>
                <button
                  onClick={editingEvent ? handleUpdateEvent : handleCreateEvent}
                  disabled={!isFormValid(eventErrors, eventForm)}
                  className="flex-1 px-4 py-2 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition disabled:bg-gray-400 disabled:cursor-not-allowed"
                >
                  {editingEvent ? "Update" : "Create"}
                </button>
              </div>
            </div>
          </div>
        )}

        {showPackageModal && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-white rounded-xl shadow-2xl p-8 w-full max-w-md max-h-[90vh] overflow-y-auto">
              <h2 className="text-2xl font-bold text-gray-900 mb-6">
                {editingPackage ? "Edit Package" : "Create Package"}
              </h2>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Name <span className="text-red-500">*</span>
                  </label>
                  <input
                    type="text"
                    value={packageForm.nume}
                    onChange={(e) => setPackageForm({ ...packageForm, nume: e.target.value })}
                    maxLength={100}
                    className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 ${packageErrors.nume ? "border-red-500" : "border-gray-300"
                      }`}
                  />
                  {renderValidationMessage(packageErrors.nume, packageForm.nume.length, 100)}
                </div>
                {renderLocationInput("package")}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
                  <textarea
                    value={packageForm.descriere}
                    onChange={(e) => setPackageForm({ ...packageForm, descriere: e.target.value })}
                    rows={3}
                    maxLength={500}
                    placeholder="Min 10 characters if provided"
                    className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 ${packageErrors.descriere ? "border-red-500" : "border-gray-300"
                      }`}
                  />
                  {renderValidationMessage(packageErrors.descriere, packageForm.descriere?.length, 500)}
                </div>
              </div>
              <div className="flex gap-3 mt-6">
                <button
                  onClick={() => {
                    setShowPackageModal(false);
                    setEditingPackage(null);
                    setLocationSuggestions([]);
                  }}
                  className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition"
                >
                  Cancel
                </button>
                <button
                  onClick={editingPackage ? handleUpdatePackage : handleCreatePackage}
                  disabled={!isFormValid(packageErrors, packageForm)}
                  className="flex-1 px-4 py-2 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition disabled:bg-gray-400 disabled:cursor-not-allowed"
                >
                  {editingPackage ? "Update" : "Create"}
                </button>
              </div>
            </div>
          </div>
        )}

        {showBindEventsModal && bindingPackage && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-white rounded-xl shadow-2xl p-8 w-full max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
              <h2 className="text-2xl font-bold text-gray-900 mb-2">
                Manage Events
              </h2>
              <p className="text-indigo-600 font-medium mb-6">Package: {bindingPackage.nume}</p>

              {bindingLoading ? (
                <div className="flex items-center justify-center py-8">
                  <div className="inline-block animate-spin rounded-full h-8 w-8 border-t-4 border-b-4 border-indigo-600"></div>
                </div>
              ) : (
                <div className="overflow-y-auto flex-1 -mx-2 px-2">
                  {events.length === 0 ? (
                    <div className="text-center py-8">
                      <p className="text-gray-600">You haven't created any events yet. Create events first to add them to this package.</p>
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {events.map((event) => {
                        const isBound = isEventBound(event.id);
                        return (
                          <div
                            key={event.id}
                            className={`rounded-lg p-4 border flex items-center justify-between ${isBound ? "bg-purple-50 border-purple-200" : "bg-gray-50 border-gray-200"
                              }`}
                          >
                            <div>
                              <p className="font-bold text-gray-900">{event.nume}</p>
                              <p className="text-sm text-gray-600">{event.locatie || "No location"}</p>
                              <p className="text-xs text-gray-500">Seats: {event.numarlocuri ?? 0}</p>
                            </div>
                            <button
                              onClick={() => isBound ? handleUnbindEvent(event.id) : handleBindEvent(event.id)}
                              disabled={bindingLoading}
                              className={`px-4 py-2 text-sm font-bold rounded-lg transition ${isBound
                                ? "text-red-600 bg-red-50 hover:bg-red-100"
                                : "text-green-600 bg-green-50 hover:bg-green-100"
                                } disabled:opacity-50`}
                            >
                              {isBound ? "Remove" : "Add"}
                            </button>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              )}

              <div className="mt-6 pt-4 border-t border-gray-200">
                <p className="text-sm text-gray-500 mb-4">
                  Events in package: {packageEvents.length}
                </p>
                <button
                  onClick={() => {
                    setShowBindEventsModal(false);
                    setBindingPackage(null);
                    setPackageEvents([]);
                  }}
                  className="w-full px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition"
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        )}

        {showTicketsModal && viewingTickets && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div className="bg-white rounded-xl shadow-2xl p-8 w-full max-w-lg max-h-[80vh] overflow-hidden flex flex-col">
              <h2 className="text-2xl font-bold text-gray-900 mb-2">
                Sold Tickets
              </h2>
              <p className="text-indigo-600 font-medium mb-6">{viewingTickets.name}</p>

              {ticketsLoading ? (
                <div className="flex items-center justify-center py-8">
                  <div className="inline-block animate-spin rounded-full h-8 w-8 border-t-4 border-b-4 border-indigo-600"></div>
                </div>
              ) : tickets.length === 0 ? (
                <div className="text-center py-8">
                  <p className="text-gray-600">No tickets sold yet.</p>
                </div>
              ) : (
                <div className="overflow-y-auto flex-1 -mx-2 px-2">
                  <div className="space-y-3">
                    {tickets.map((ticket) => (
                      <div key={ticket.cod} className="bg-gray-50 rounded-lg p-4 border border-gray-200">
                        <p className="font-mono text-sm font-bold text-indigo-600 mb-2">{ticket.cod}</p>
                        {ticket.buyerLoading ? (
                          <div className="flex items-center gap-2 text-gray-500 text-sm">
                            <div className="animate-spin rounded-full h-3 w-3 border-2 border-gray-400 border-t-transparent"></div>
                            Loading buyer info...
                          </div>
                        ) : ticket.buyer ? (
                          <div className="text-sm space-y-1">
                            <p className="text-gray-700">
                              <span className="font-medium">Email:</span> {ticket.buyer.email}
                            </p>
                            {ticket.buyer.public_info && (ticket.buyer.prenume || ticket.buyer.nume) && (
                              <p className="text-gray-700">
                                <span className="font-medium">Name:</span>{" "}
                                {[ticket.buyer.prenume, ticket.buyer.nume].filter(Boolean).join(" ")}
                              </p>
                            )}
                            {!ticket.buyer.public_info && (
                              <p className="text-gray-400 text-xs italic">Profile is private</p>
                            )}
                          </div>
                        ) : (
                          <p className="text-gray-400 text-sm italic">Buyer info unavailable</p>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              <div className="mt-6 pt-4 border-t border-gray-200">
                <p className="text-sm text-gray-500 mb-4">
                  Total tickets sold: {tickets.length}
                </p>
                <button
                  onClick={() => {
                    setShowTicketsModal(false);
                    setViewingTickets(null);
                    setTickets([]);
                  }}
                  className="w-full px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition"
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        )}

        <ConfirmModal
          isOpen={deleteConfirm !== null}
          title={`Delete ${deleteConfirm?.type === "event" ? "Event" : "Package"}`}
          message={`Are you sure you want to delete "${deleteConfirm?.name}"? This action cannot be undone.`}
          confirmText="Delete"
          cancelText="Cancel"
          onConfirm={confirmDelete}
          onCancel={() => setDeleteConfirm(null)}
        />

        <ErrorModal
          isOpen={errorModal !== null}
          title={errorModal?.title || "Error"}
          message={errorModal?.message || "An error occurred"}
          onClose={() => setErrorModal(null)}
        />
      </div>
    </div>
  );
}
