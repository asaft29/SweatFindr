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
              <div key={ticket.cod} className="relative bg-white rounded-xl shadow-lg overflow-hidden hover:shadow-2xl transition-shadow duration-300">
                <div className="absolute left-0 top-0 bottom-0 w-3 bg-gradient-to-b from-indigo-500 to-indigo-700"></div>
                <div className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 bg-gradient-to-br from-blue-50 to-indigo-100 rounded-full -ml-2"></div>

                <div className="pl-6 pr-6 py-6">
                  <div className="flex items-center justify-between mb-4 pb-4 border-b border-dashed border-gray-300">
                    <div>
                      <p className="text-xs text-gray-500 uppercase tracking-wider">Ticket Code</p>
                      <p className="text-lg font-mono font-bold text-indigo-600">{ticket.cod}</p>
                    </div>
                    <div className="text-right">
                      <div className="w-12 h-12 bg-indigo-100 rounded-lg flex items-center justify-center">
                        <svg className="w-6 h-6 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z" />
                        </svg>
                      </div>
                    </div>
                  </div>

                  {ticket.nume_eveniment && (
                    <h3 className="text-xl font-bold text-gray-900 mb-3">{ticket.nume_eveniment}</h3>
                  )}

                  <div className="space-y-2">
                    {ticket.locatie && (
                      <div className="flex items-center text-gray-600">
                        <svg className="w-4 h-4 mr-2 text-indigo-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                        </svg>
                        <span className="text-sm">{ticket.locatie}</span>
                      </div>
                    )}
                    {ticket.descriere && (
                      <div className="flex items-start text-gray-600">
                        <svg className="w-4 h-4 mr-2 mt-0.5 text-indigo-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <span className="text-sm">{ticket.descriere}</span>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
