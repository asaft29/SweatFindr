import { useEffect, useState } from "react";
import { eventService } from "../lib/eventService";
import type { Event } from "../lib/types";

export function EventsPage() {
  const [events, setEvents] = useState<Event[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [locatieFilter, setLocatieFilter] = useState("");
  const [numeFilter, setNumeFilter] = useState("");

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

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
        <div className="text-xl font-semibold text-indigo-600">Loading events...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
        <div className="bg-red-50 border border-red-200 text-red-800 px-6 py-4 rounded-lg">{error}</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-4xl font-extrabold text-gray-900 mb-8">Events</h1>

        <div className="bg-white p-6 rounded-xl shadow-lg mb-8">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <input
              type="text"
              placeholder="Filter by location"
              value={locatieFilter}
              onChange={(e) => setLocatieFilter(e.target.value)}
              className="appearance-none rounded-lg px-4 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition"
            />
            <input
              type="text"
              placeholder="Filter by name"
              value={numeFilter}
              onChange={(e) => setNumeFilter(e.target.value)}
              className="appearance-none rounded-lg px-4 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition"
            />
            <button onClick={handleFilter} className="px-6 py-2 rounded-lg text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition font-medium">
              Apply Filters
            </button>
            <button onClick={handleClearFilters} className="px-6 py-2 rounded-lg text-indigo-600 bg-white border border-indigo-600 hover:bg-indigo-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition font-medium">
              Clear Filters
            </button>
          </div>
        </div>

        {events.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-xl text-gray-600">No events found.</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {events.map((event) => (
              <div key={event.id} className="bg-white rounded-xl shadow-lg p-6 hover:shadow-2xl transition-shadow duration-300">
                <h3 className="text-2xl font-bold text-gray-900 mb-3">{event.nume}</h3>
                <p className="text-indigo-600 font-medium mb-2">{event.locatie || "Location not specified"}</p>
                <p className="text-gray-700 mb-4">{event.descriere || "No description available"}</p>
                <p className="text-sm text-gray-500 font-medium">
                  Available seats: {event.numarlocuri !== null ? event.numarlocuri : "N/A"}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
