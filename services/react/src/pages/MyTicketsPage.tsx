import { useEffect, useState, useMemo } from "react";
import { clientService } from "../lib/clientService";
import type { TicketRef } from "../lib/types";

interface GroupedTickets {
  name: string;
  locatie?: string;
  descriere?: string;
  tickets: TicketRef[];
}

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

  const groupedTickets = useMemo(() => {
    const groups: Map<string, GroupedTickets> = new Map();

    for (const ticket of tickets) {
      const key = ticket.nume_eveniment || "Unknown Event";

      if (!groups.has(key)) {
        groups.set(key, {
          name: key,
          locatie: ticket.locatie,
          descriere: ticket.descriere,
          tickets: [],
        });
      }

      groups.get(key)!.tickets.push(ticket);
    }

    return Array.from(groups.values());
  }, [tickets]);

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

        {!loading && !error && groupedTickets.length > 0 && (
          <div className="space-y-8">
            {groupedTickets.map((group) => (
              <div key={group.name} className="bg-white rounded-xl shadow-lg overflow-hidden">
                <div className="bg-gradient-to-r from-indigo-600 to-indigo-700 px-6 py-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <h3 className="text-xl font-bold text-white">{group.name}</h3>
                      {group.locatie && (
                        <div className="flex items-center text-indigo-100 mt-1">
                          <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                          </svg>
                          <span className="text-sm">{group.locatie}</span>
                        </div>
                      )}
                    </div>
                    <div className="bg-white bg-opacity-20 rounded-lg px-3 py-1">
                      <span className="text-white font-bold">{group.tickets.length} ticket{group.tickets.length !== 1 ? 's' : ''}</span>
                    </div>
                  </div>
                  {group.descriere && (
                    <p className="text-indigo-100 text-sm mt-2">{group.descriere}</p>
                  )}
                </div>

                <div className="p-4">
                  <div className="overflow-x-auto pb-2">
                    <div className="flex gap-3 min-w-min">
                      {group.tickets.map((ticket) => (
                        <div
                          key={ticket.cod}
                          className="relative bg-gray-50 rounded-lg p-4 border border-gray-200 hover:border-indigo-300 hover:shadow-md transition-all duration-200 flex-shrink-0 w-64"
                        >
                          <div className="absolute left-0 top-0 bottom-0 w-1 bg-indigo-500 rounded-l-lg"></div>
                          <div className="pl-2">
                            <p className="text-xs text-gray-500 uppercase tracking-wider">Ticket Code</p>
                            <p className="text-sm font-mono font-bold text-indigo-600 break-all">{ticket.cod}</p>
                          </div>
                        </div>
                      ))}
                    </div>
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
