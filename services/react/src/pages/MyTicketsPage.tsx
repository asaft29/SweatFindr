import { useEffect, useState } from "react";
import { clientService } from "../lib/clientService";
import type { TicketRef } from "../lib/types";

export function MyTicketsPage() {
  const [tickets, setTickets] = useState<TicketRef[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadTickets = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await clientService.getMyTickets();
      setTickets(data);
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || "Failed to load tickets");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadTickets();
  }, []);

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-4xl font-extrabold text-gray-900 mb-8">My Tickets</h1>

        {loading && (
          <div className="flex items-center justify-center py-12">
            <div className="inline-block animate-spin rounded-full h-12 w-12 border-t-4 border-b-4 border-indigo-600"></div>
          </div>
        )}

        {error && (
          <div className="text-center py-12">
            <div className="bg-red-50 border border-red-200 text-red-800 px-6 py-4 rounded-lg inline-block">
              {error}
            </div>
          </div>
        )}

        {!loading && !error && tickets.length === 0 && (
          <div className="text-center py-12">
            <p className="text-xl text-gray-600">You haven't purchased any tickets yet.</p>
          </div>
        )}

        {!loading && !error && tickets.length > 0 && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {tickets.map((ticket) => (
              <div key={ticket.cod} className="bg-white rounded-xl shadow-lg p-6">
                <h3 className="text-2xl font-bold text-gray-900 mb-3">Ticket #{ticket.cod}</h3>
                <div className="space-y-2">
                  {ticket.nume_eveniment && (
                    <p className="text-gray-700">
                      <span className="font-medium">Event:</span> {ticket.nume_eveniment}
                    </p>
                  )}
                  {ticket.locatie && (
                    <p className="text-gray-700">
                      <span className="font-medium">Location:</span> {ticket.locatie}
                    </p>
                  )}
                  {ticket.descriere && (
                    <p className="text-gray-700">
                      <span className="font-medium">Description:</span> {ticket.descriere}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
