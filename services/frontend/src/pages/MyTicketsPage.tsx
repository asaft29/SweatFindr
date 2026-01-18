import { useEffect, useState, useMemo, useCallback } from "react";
import { clientService } from "../lib/clientService";
import type { TicketRef } from "../lib/types";
import { SuccessModal } from "../components/SuccessModal";
import { ErrorModal } from "../components/ErrorModal";
import { useRefundNotifications } from "../hooks/useRefundNotifications";

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
  const [refundModal, setRefundModal] = useState<{ ticketCode: string; eventName: string } | null>(null);
  const [refundReason, setRefundReason] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadTickets = useCallback(async (showLoading = true) => {
    try {
      if (showLoading) setLoading(true);
      setError(null);
      const data = await clientService.getMyTickets();
      setTickets(data);
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || "Failed to load tickets");
      console.error(err);
    } finally {
      if (showLoading) setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadTickets();
  }, [loadTickets]);

  useRefundNotifications({
    onRefundStatusChanged: useCallback(() => {
      loadTickets(false);
    }, [loadTickets]),
  });

  const handleRefundRequest = async () => {
    if (!refundModal || !refundReason.trim()) return;

    setSubmitting(true);
    try {
      const response = await clientService.requestRefund(refundModal.ticketCode, refundReason);
      setSuccessMessage(response.message || "Refund request submitted successfully!");
      setRefundModal(null);
      setRefundReason("");
      await loadTickets();
    } catch (err: any) {
      setErrorMessage(err.response?.data?.error || "Failed to submit refund request");
    } finally {
      setSubmitting(false);
    }
  };

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
                      {group.tickets.map((ticket) => {
                        const isPending = ticket.refund_status === "PENDING";
                        const isRefunded = ticket.refund_status === "APPROVED";
                        const isRejected = ticket.refund_status === "REJECTED";
                        const canRequestRefund = !isPending && !isRefunded && !isRejected;

                        return (
                          <div
                            key={ticket.cod}
                            className={`relative bg-gray-50 rounded-lg p-4 border border-gray-200 hover:border-indigo-300 hover:shadow-md transition-all duration-200 flex-shrink-0 w-64 ${isPending ? 'border-yellow-300 bg-yellow-50' : ''} ${isRefunded ? 'border-green-300 bg-green-50' : ''} ${isRejected ? 'border-red-300 bg-red-50' : ''}`}
                          >
                            <div className={`absolute left-0 top-0 bottom-0 w-1 rounded-l-lg ${isPending ? 'bg-yellow-500' : isRefunded ? 'bg-green-500' : isRejected ? 'bg-red-500' : 'bg-indigo-500'}`}></div>
                            <div className="pl-2">
                              <p className="text-xs text-gray-500 uppercase tracking-wider">Ticket Code</p>
                              <p className="text-sm font-mono font-bold text-indigo-600 break-all mb-2">{ticket.cod}</p>
                              {isPending && (
                                <span className="inline-block text-xs px-2 py-1 bg-yellow-100 text-yellow-700 rounded-full font-medium mb-2">
                                  Pending
                                </span>
                              )}
                              {isRefunded && (
                                <span className="inline-block text-xs px-2 py-1 bg-green-100 text-green-700 rounded-full font-medium mb-2">
                                  Refunded
                                </span>
                              )}
                              {isRejected && (
                                <span className="inline-block text-xs px-2 py-1 bg-red-100 text-red-700 rounded-full font-medium mb-2">
                                  Rejected
                                </span>
                              )}
                              {canRequestRefund && (
                                <button
                                  onClick={() => setRefundModal({ ticketCode: ticket.cod, eventName: group.name })}
                                  className="text-xs px-3 py-1.5 bg-red-50 text-red-600 rounded-lg hover:bg-red-100 transition-colors font-medium"
                                >
                                  Request Refund
                                </button>
                              )}
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Refund Request Modal */}
      {refundModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-2xl p-6 max-w-md w-full mx-4 shadow-2xl">
            <h3 className="text-xl font-bold text-gray-800 mb-2">Request Refund</h3>
            <p className="text-gray-600 mb-1">
              Event: <span className="font-medium">{refundModal.eventName}</span>
            </p>
            <p className="text-gray-500 text-sm mb-4">
              Ticket: <span className="font-mono">{refundModal.ticketCode}</span>
            </p>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Reason for refund
            </label>
            <textarea
              value={refundReason}
              onChange={(e) => setRefundReason(e.target.value)}
              className="w-full border border-gray-300 rounded-lg p-3 mb-4 focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
              rows={3}
              placeholder="Please explain why you're requesting a refund..."
            />
            <div className="flex gap-3 justify-end">
              <button
                onClick={() => {
                  setRefundModal(null);
                  setRefundReason("");
                }}
                className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                disabled={submitting}
              >
                Cancel
              </button>
              <button
                onClick={handleRefundRequest}
                disabled={!refundReason.trim() || submitting}
                className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {submitting ? "Submitting..." : "Submit Request"}
              </button>
            </div>
          </div>
        </div>
      )}

      <SuccessModal
        isOpen={!!successMessage}
        title="Refund Request Submitted"
        message={successMessage || ""}
        onClose={() => setSuccessMessage(null)}
      />

      <ErrorModal
        isOpen={!!errorMessage}
        title="Error"
        message={errorMessage || ""}
        onClose={() => setErrorMessage(null)}
      />
    </div>
  );
}
