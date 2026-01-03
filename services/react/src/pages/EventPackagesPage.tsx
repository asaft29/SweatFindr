import { useEffect, useState } from "react";
import { eventService } from "../lib/eventService";
import { clientService } from "../lib/clientService";
import { useAuthStore } from "../lib/useAuthStore";
import { ConfirmModal } from "../components/ConfirmModal";
import { SuccessModal } from "../components/SuccessModal";
import type { EventPackageWithLinks } from "../lib/types";

export function EventPackagesPage() {
  const { user } = useAuthStore();
  const [packages, setPackages] = useState<EventPackageWithLinks[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [purchasing, setPurchasing] = useState<number | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage, setItemsPerPage] = useState(3);
  const [itemsPerPageInput, setItemsPerPageInput] = useState("3");
  const [filters, setFilters] = useState({
    type: "",
    availableTickets: "",
  });
  const [nextLink, setNextLink] = useState<string | null>(null);
  const [prevLink, setPrevLink] = useState<string | null>(null);
  const [showSuccess, setShowSuccess] = useState(false);
  const [purchaseConfirm, setPurchaseConfirm] = useState<EventPackageWithLinks | null>(null);

  const loadPackages = async () => {
    try {
      setLoading(true);
      setError(null);

      if (filters.type.trim() && filters.type.trim().length < 3) {
        setError("Description must be at least 3 characters");
        setLoading(false);
        return;
      }

      const filterParams: any = {};
      if (filters.type.trim()) filterParams.type = filters.type.trim();
      if (filters.availableTickets.trim()) {
        filterParams.availableTickets = parseInt(filters.availableTickets);
      }
      filterParams.page = currentPage;
      filterParams.itemsPerPage = itemsPerPage;
      const data = await eventService.getEventPackages(filterParams);

      if (data.length === 0 && currentPage > 1) {
        setCurrentPage(currentPage - 1);
        return;
      }

      setPackages(data);

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
        setError("Failed to load event packages");
        console.error(err);
      }
    } finally {
      setLoading(false);
    }
  };

  const loadPackagesByUrl = async (url: string) => {
    try {
      setLoading(true);
      setError(null);
      const data = await eventService.getEventPackagesByUrl(url);

      setPackages(data);

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
        setCurrentPage(parseInt(pageParam));
      }
    } catch (err: any) {
      setError("Failed to load event packages");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handlePreviousPage = () => {
    if (prevLink) {
      loadPackagesByUrl(prevLink);
    }
  };

  const handleNextPage = () => {
    if (nextLink) {
      loadPackagesByUrl(nextLink);
    }
  };

  const handlePurchase = async (packageId: number) => {
    if (user?.role !== 'client') {
      setError("Only clients can purchase tickets");
      return;
    }

    try {
      setPurchasing(packageId);
      setError(null);
      await clientService.purchaseTicket({ pachetid: packageId });
      setShowSuccess(true);
      await loadPackages();
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || "Failed to purchase ticket");
      console.error(err);
    } finally {
      setPurchasing(null);
    }
  };

  useEffect(() => {
    loadPackages();
  }, [currentPage, itemsPerPage]);

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <div className="bg-white p-6 rounded-xl shadow-lg mb-8">
          <h2 className="text-xl font-bold text-gray-900 mb-4">Filter Packages</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div>
              <label htmlFor="type" className="block text-sm font-medium text-gray-700 mb-1">
                Description
              </label>
              <input
                id="type"
                type="text"
                autoComplete="off"
                value={filters.type}
                onChange={(e) => {
                  const value = e.target.value;
                  if (value.length <= 50) {
                    setFilters({ ...filters, type: value });
                  }
                }}
                placeholder="Search by package type (3-50 chars)..."
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
              />
            </div>
            <div>
              <label htmlFor="availableTickets" className="block text-sm font-medium text-gray-700 mb-1">
                Minimum Available Tickets
              </label>
              <input
                id="availableTickets"
                type="text"
                inputMode="numeric"
                autoComplete="off"
                value={filters.availableTickets}
                onChange={(e) => {
                  const value = e.target.value;
                  if (value === '' || /^\d+$/.test(value)) {
                    setFilters({ ...filters, availableTickets: value });
                  }
                }}
                placeholder="Min tickets..."
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
                const value = parseInt(itemsPerPageInput);
                if (!isNaN(value) && value >= 1) {
                  const cappedValue = Math.min(value, 100);
                  setItemsPerPage(cappedValue);
                  setItemsPerPageInput(cappedValue.toString());
                  setCurrentPage(1);
                } else {
                  setItemsPerPageInput(itemsPerPage.toString());
                }
              }}
              className="px-6 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition shadow-md hover:shadow-lg"
            >
              Search
            </button>
            <button
              onClick={() => {
                setFilters({ type: "", availableTickets: "" });
                setCurrentPage(1);
                setItemsPerPage(3);
                setItemsPerPageInput("3");
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
          ) : packages.length === 0 ? (
            <div className="text-center py-12 animate-fade-in">
              <p className="text-xl text-gray-600">No event packages found.</p>
            </div>
          ) : (
            <div className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 animate-fade-in ${loading ? 'opacity-50' : ''}`}>
              {packages.map((pkg) => (
                <div key={pkg.id} className="bg-white rounded-xl shadow-lg p-6 hover:shadow-2xl transition-shadow duration-300 flex flex-col h-full">
                  <h3 className="text-2xl font-bold text-gray-900 mb-3">{pkg.nume}</h3>
                  <p className="text-indigo-600 font-medium mb-2">{pkg.locatie || "Location not specified"}</p>
                  <p className="text-gray-700 mb-4 flex-grow">{pkg.descriere || "No description available"}</p>
                  <div className="mt-auto">
                    {pkg.numarlocuri !== null && pkg.numarlocuri > 0 && (
                      <p className="text-sm text-gray-500 font-medium mb-4">
                        Available seats: {pkg.numarlocuri}
                      </p>
                    )}
                    {pkg.numarlocuri !== null && pkg.numarlocuri > 0 ? (
                      user?.role === 'client' ? (
                        <button
                          onClick={() => setPurchaseConfirm(pkg)}
                          disabled={purchasing === pkg.id}
                          className="w-full px-4 py-2 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition disabled:bg-gray-400"
                        >
                          {purchasing === pkg.id ? "Purchasing..." : "Buy Ticket"}
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

        {!loading && !error && (packages.length > 0 || currentPage > 1) && (
          <div className="w-full flex items-center justify-center gap-6 mt-8">
            <button
              onClick={handlePreviousPage}
              disabled={!prevLink}
              className={`w-12 h-12 flex items-center justify-center rounded-full transition shadow-md hover:shadow-lg ${
                !prevLink
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
              className={`w-12 h-12 flex items-center justify-center rounded-full transition shadow-md hover:shadow-lg ${
                !nextLink
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
