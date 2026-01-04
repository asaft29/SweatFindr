import { useEffect, useState } from "react";
import { eventService } from "../lib/eventService";
import { clientService } from "../lib/clientService";
import { useAuthStore } from "../lib/useAuthStore";
import { ConfirmModal } from "../components/ConfirmModal";
import { SuccessModal } from "../components/SuccessModal";
import type { Event } from "../lib/types";

export function EventsPage() {
  const { user } = useAuthStore();
  const [events, setEvents] = useState<Event[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [purchasing, setPurchasing] = useState<number | null>(null);
  const [locatieFilter, setLocatieFilter] = useState("");
  const [numeFilter, setNumeFilter] = useState("");
  const [showSuccess, setShowSuccess] = useState(false);
  const [purchaseConfirm, setPurchaseConfirm] = useState<Event | null>(null);

  useEffect(() => {
    loadEvents();
  }, []);

  const loadEvents = async (filters?: { locatie?: string; nume?: string }) => {
    try {
      setLoading(true);
      setError(null);
      const data = await eventService.getEvents(filters);
      setEvents(data);
    } catch (err) {
      setError("Failed to load events");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleFilter = () => {
    const filters: { locatie?: string; nume?: string } = {};
    if (locatieFilter) filters.locatie = locatieFilter;
    if (numeFilter) filters.nume = numeFilter;
    loadEvents(filters);
  };

  const handleClearFilters = () => {
    setLocatieFilter("");
    setNumeFilter("");
    loadEvents();
  };

  const handlePurchase = async (eventId: number) => {
    if (user?.role !== 'client') {
      setError("Only clients can purchase tickets");
      return;
    }

    try {
      setPurchasing(eventId);
      setError(null);
      await clientService.purchaseTicket({ evenimentid: eventId });
      setShowSuccess(true);
      await loadEvents();
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || "Failed to purchase ticket");
      console.error(err);
    } finally {
      setPurchasing(null);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <div className="bg-white p-6 rounded-xl shadow-lg mb-8">
          <h2 className="text-xl font-bold text-gray-900 mb-4">Filter Events</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label htmlFor="location" className="block text-sm font-medium text-gray-700 mb-1">
                Location
              </label>
              <input
                id="location"
                type="text"
                autoComplete="off"
                placeholder="Search by location (max 50 chars)..."
                value={locatieFilter}
                onChange={(e) => {
                  const value = e.target.value;
                  if (value.length <= 50) {
                    setLocatieFilter(value);
                  }
                }}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
              />
            </div>
            <div>
              <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-1">
                Event Name
              </label>
              <input
                id="name"
                type="text"
                autoComplete="off"
                placeholder="Search by name (max 50 chars)..."
                value={numeFilter}
                onChange={(e) => {
                  const value = e.target.value;
                  if (value.length <= 50) {
                    setNumeFilter(value);
                  }
                }}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
              />
            </div>
          </div>
          <div className="flex gap-3 mt-4">
            <button
              onClick={handleFilter}
              className="px-6 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition shadow-md hover:shadow-lg"
            >
              Search
            </button>
            <button
              onClick={handleClearFilters}
              className="px-4 py-2 text-sm font-medium text-indigo-600 hover:text-indigo-700 hover:bg-indigo-50 rounded-lg transition"
            >
              Clear Filters
            </button>
          </div>
        </div>

        <div className="relative min-h-[200px]">
          {loading && (
            <div className="absolute inset-0 flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 bg-opacity-50 z-10">
              <div className="inline-block animate-spin rounded-full h-12 w-12 border-t-4 border-b-4 border-indigo-600"></div>
            </div>
          )}
          {error ? (
            <div className="text-center py-12 animate-fade-in">
              <div className="bg-red-50 border border-red-200 text-red-800 px-6 py-4 rounded-lg inline-block">{error}</div>
            </div>
          ) : events.length === 0 ? (
            <div className="text-center py-12 animate-fade-in">
              <p className="text-xl text-gray-600">No events found.</p>
            </div>
          ) : (
            <div className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 animate-fade-in ${loading ? 'opacity-50' : ''}`}>
              {events.map((event) => (
                <div key={event.id} className="bg-white rounded-xl shadow-lg p-6 hover:shadow-2xl transition-shadow duration-300 flex flex-col h-full">
                  <h3 className="text-2xl font-bold text-gray-900 mb-3">{event.nume}</h3>
                  <p className="text-indigo-600 font-medium mb-2">{event.locatie || "Location not specified"}</p>
                  <p className="text-gray-700 mb-4 flex-grow">{event.descriere || "No description available"}</p>
                  <div className="mt-auto">
                    {event.numarlocuri !== null && event.numarlocuri > 0 && (
                      <p className="text-sm text-gray-500 font-medium mb-4">
                        Available seats: {event.numarlocuri}
                      </p>
                    )}
                    {event.numarlocuri !== null && event.numarlocuri > 0 ? (
                      user?.role === 'client' ? (
                        <button
                          onClick={() => setPurchaseConfirm(event)}
                          disabled={purchasing === event.id}
                          className="w-full px-4 py-2 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition disabled:bg-gray-400"
                        >
                          {purchasing === event.id ? "Purchasing..." : "Buy Ticket"}
                        </button>
                      ) : null
                    ) : (
                      <button
                        disabled
                        className="w-full px-4 py-2 text-sm font-bold text-white bg-red-500 rounded-lg cursor-not-allowed"
                      >
                        Sold Out
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <SuccessModal
          isOpen={showSuccess}
          title="Purchase Complete"
          message="Your ticket has been purchased successfully!"
          onClose={() => setShowSuccess(false)}
        />

        <ConfirmModal
          isOpen={purchaseConfirm !== null}
          title="Confirm Purchase"
          message={`Are you sure you want to buy a ticket for "${purchaseConfirm?.nume}"?`}
          confirmText="Buy Ticket"
          cancelText="Cancel"
          confirmButtonClass="bg-indigo-600 hover:bg-indigo-700"
          onConfirm={() => {
            if (purchaseConfirm) {
              handlePurchase(purchaseConfirm.id);
              setPurchaseConfirm(null);
            }
          }}
          onCancel={() => setPurchaseConfirm(null)}
        />
      </div>
    </div>
  );
}
