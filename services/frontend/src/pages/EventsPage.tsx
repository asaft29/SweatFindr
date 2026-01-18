import { useEffect, useState, useRef } from "react";
import { eventService } from "../lib/eventService";
import { clientService } from "../lib/clientService";
import { useAuthStore } from "../lib/useAuthStore";
import { ConfirmModal } from "../components/ConfirmModal";
import { SuccessModal } from "../components/SuccessModal";
import type { EventWithLinks } from "../lib/types";

export function EventsPage() {
  const { user } = useAuthStore();
  const [events, setEvents] = useState<EventWithLinks[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [purchasing, setPurchasing] = useState<number | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage, setItemsPerPage] = useState(10);
  const [itemsPerPageInput, setItemsPerPageInput] = useState("10");
  const [filters, setFilters] = useState({
    locatie: "",
    nume: "",
  });
  const [nextLink, setNextLink] = useState<string | null>(null);
  const [prevLink, setPrevLink] = useState<string | null>(null);
  const [currentLink, setCurrentLink] = useState<string | null>(null);
  const [showSuccess, setShowSuccess] = useState(false);
  const [purchaseConfirm, setPurchaseConfirm] = useState<EventWithLinks | null>(null);
  const isHateoasNavigation = useRef(false);
  const [searchTrigger, setSearchTrigger] = useState(0);

  const loadEvents = async () => {
    try {
      setLoading(true);
      setError(null);

      const filterParams: { locatie?: string; nume?: string; page?: number; itemsPerPage?: number } = {};
      if (filters.locatie.trim()) filterParams.locatie = filters.locatie.trim();
      if (filters.nume.trim()) filterParams.nume = filters.nume.trim();
      filterParams.page = currentPage;
      filterParams.itemsPerPage = itemsPerPage;
      const data = await eventService.getEvents(filterParams);

      if (data.length === 0 && currentPage > 1) {
        setCurrentPage(currentPage - 1);
        return;
      }

      setEvents(data);
      setCurrentLink(null);

      if (data.length > 0 && data[0]._links) {
        setNextLink(data[0]._links.next?.href || null);
        setPrevLink(data[0]._links.prev?.href || null);
      } else {
        setNextLink(null);
        setPrevLink(null);
      }
    } catch (err: any) {
      if (err.response?.status === 422) {
        setError("Invalid filter values. Items per page must be between 1 and 100.");
      } else {
        setError("Failed to load events");
        console.error(err);
      }
    } finally {
      setLoading(false);
    }
  };

  const loadEventsByUrl = async (url: string) => {
    try {
      setLoading(true);
      setError(null);
      const data = await eventService.getEventsByUrl(url);

      setEvents(data);
      setCurrentLink(url);

      if (data.length > 0 && data[0]._links) {
        setNextLink(data[0]._links.next?.href || null);
        setPrevLink(data[0]._links.prev?.href || null);
      } else {
        setNextLink(null);
        setPrevLink(null);
      }

      const urlObj = new URL(url, window.location.origin);
      const pageParam = urlObj.searchParams.get('page');
      if (pageParam) {
        isHateoasNavigation.current = true;
        setCurrentPage(parseInt(pageParam));
      }
    } catch (err: any) {
      setError("Failed to load events");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handlePreviousPage = () => {
    if (prevLink) {
      loadEventsByUrl(prevLink);
    }
  };

  const handleNextPage = () => {
    if (nextLink) {
      loadEventsByUrl(nextLink);
    }
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

      if (currentLink) {
        await loadEventsByUrl(currentLink);
      } else {
        await loadEvents();
      }
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || "Failed to purchase ticket");
      console.error(err);
    } finally {
      setPurchasing(null);
    }
  };

  useEffect(() => {
    if (isHateoasNavigation.current) {
      isHateoasNavigation.current = false;
      return;
    }
    loadEvents();
  }, [currentPage, itemsPerPage, searchTrigger]);

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <div className="bg-white p-6 rounded-xl shadow-lg mb-8">
          <h2 className="text-xl font-bold text-gray-900 mb-4">Filter Events</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label htmlFor="location" className="block text-sm font-medium text-gray-700 mb-1">
                Location
              </label>
              <input
                id="location"
                type="text"
                autoComplete="off"
                value={filters.locatie}
                onChange={(e) => {
                  const value = e.target.value;
                  if (value.length <= 50) {
                    setFilters({ ...filters, locatie: value });
                  }
                }}
                placeholder="Search by location (max 50 chars)..."
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
                value={filters.nume}
                onChange={(e) => {
                  const value = e.target.value;
                  if (value.length <= 50) {
                    setFilters({ ...filters, nume: value });
                  }
                }}
                placeholder="Search by name (max 50 chars)..."
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
              />
            </div>
            <div>
              <label htmlFor="itemsPerPage" className="block text-sm font-medium text-gray-700 mb-1">
                Items Per Page
              </label>
              <input
                id="itemsPerPage"
                type="text"
                inputMode="numeric"
                autoComplete="off"
                value={itemsPerPageInput}
                onChange={(e) => {
                  const value = e.target.value;
                  if (value === '' || /^\d+$/.test(value)) {
                    setItemsPerPageInput(value);
                  }
                }}
                placeholder="Items per page (1-100)..."
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
              />
            </div>
          </div>
          <div className="flex gap-3 mt-4">
            <button
              onClick={() => {
                const itemsValue = parseInt(itemsPerPageInput);
                if (!isNaN(itemsValue) && itemsValue >= 1) {
                  const cappedItemsValue = Math.min(itemsValue, 100);
                  setItemsPerPage(cappedItemsValue);
                  setItemsPerPageInput(cappedItemsValue.toString());
                }
                setCurrentPage(1);
                setSearchTrigger(prev => prev + 1);
              }}
              className="px-6 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition shadow-md hover:shadow-lg"
            >
              Search
            </button>
            <button
              onClick={() => {
                setFilters({ locatie: "", nume: "" });
                setCurrentPage(1);
                setItemsPerPage(10);
                setItemsPerPageInput("10");
              }}
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

        {!loading && !error && (events.length > 0 || currentPage > 1) && (
          <div className="w-full flex items-center justify-center gap-6 mt-8">
            <button
              onClick={handlePreviousPage}
              disabled={!prevLink}
              className={`w-12 h-12 flex items-center justify-center rounded-full transition shadow-md hover:shadow-lg ${!prevLink
                ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                : 'bg-indigo-600 text-white hover:bg-indigo-700'
                }`}
              aria-label="Previous page"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <span className="text-lg font-semibold text-gray-700 min-w-[100px] text-center">
              Page {currentPage}
            </span>
            <button
              onClick={handleNextPage}
              disabled={!nextLink}
              className={`w-12 h-12 flex items-center justify-center rounded-full transition shadow-md hover:shadow-lg ${!nextLink
                ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                : 'bg-indigo-600 text-white hover:bg-indigo-700'
                }`}
              aria-label="Next page"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>
        )}

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
