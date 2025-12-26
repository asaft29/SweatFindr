import { useEffect, useState } from "react";
import { eventService } from "../lib/eventService";
import type { EventPackage } from "../lib/types";

export function EventPackagesPage() {
  const [packages, setPackages] = useState<EventPackage[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadPackages();
  }, []);

  const loadPackages = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await eventService.getEventPackages();
      setPackages(data);
    } catch (err) {
      setError("Failed to load event packages");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
        <div className="text-xl font-semibold text-indigo-600">Loading event packages...</div>
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
        <h1 className="text-4xl font-extrabold text-gray-900 mb-8">Event Packages</h1>

        {packages.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-xl text-gray-600">No event packages found.</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {packages.map((pkg) => (
              <div key={pkg.id} className="bg-white rounded-xl shadow-lg p-6 hover:shadow-2xl transition-shadow duration-300">
                <h3 className="text-2xl font-bold text-gray-900 mb-3">{pkg.nume}</h3>
                <p className="text-indigo-600 font-medium mb-2">{pkg.locatie || "Location not specified"}</p>
                <p className="text-gray-700 mb-4">{pkg.descriere || "No description available"}</p>
                <p className="text-sm text-gray-500 font-medium">
                  Available seats: {pkg.numarlocuri !== null ? pkg.numarlocuri : "N/A"}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
