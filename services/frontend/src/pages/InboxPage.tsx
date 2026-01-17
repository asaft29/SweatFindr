import { useState, useEffect, useCallback } from 'react';
import { useAuthStore } from '../lib/useAuthStore';
import { RefundRequest, UserRole } from '../lib/types';
import { getOwnerRefundRequests, approveRefund, rejectRefund, getClientRefunds } from '../lib/refundService';
import { AnimatedPage } from '../components/AnimatedPage';
import { ConfirmModal } from '../components/ConfirmModal';
import { ErrorModal } from '../components/ErrorModal';
import { SuccessModal } from '../components/SuccessModal';
import { useRefundNotifications } from '../hooks/useRefundNotifications';

export const InboxPage = () => {
    const { user } = useAuthStore();
    const [refunds, setRefunds] = useState<RefundRequest[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);
    const [confirmAction, setConfirmAction] = useState<{ id: number; action: 'approve' | 'reject' } | null>(null);
    const [rejectReason, setRejectReason] = useState('');

    const isOwner = user?.role === UserRole.OWNER_EVENT;
    const isClient = user?.role === UserRole.CLIENT;

    const loadRefunds = useCallback(async (showLoading = true) => {
        if (!user) return;

        if (showLoading) setLoading(true);
        setError(null);

        try {
            if (isOwner) {
                const data = await getOwnerRefundRequests();
                setRefunds(data);
            } else if (isClient && user.email) {
                const data = await getClientRefunds(user.email);
                setRefunds(data);
            }
        } catch (err) {
            console.error('Failed to load refunds:', err);
            setError('Failed to load refund requests');
        } finally {
            if (showLoading) setLoading(false);
        }
    }, [user, isOwner, isClient]);

    useEffect(() => {
        loadRefunds();
    }, [loadRefunds]);

    useRefundNotifications({
        onRefundStatusChanged: useCallback(() => {
            if (isClient) loadRefunds(false);
        }, [isClient, loadRefunds]),
        onNewRefundRequest: useCallback(() => {
            if (isOwner) loadRefunds(false);
        }, [isOwner, loadRefunds]),
    });

    const handleApprove = async (id: number) => {
        try {
            await approveRefund(id);
            setSuccessMessage('Refund approved successfully! The client will be notified.');
            setConfirmAction(null);
            loadRefunds();
        } catch (err) {
            setError('Failed to approve refund');
        }
    };

    const handleReject = async (id: number, reason: string) => {
        try {
            await rejectRefund(id, reason);
            setSuccessMessage('Refund rejected. The client will be notified.');
            setConfirmAction(null);
            setRejectReason('');
            loadRefunds();
        } catch (err) {
            setError('Failed to reject refund');
        }
    };

    const getStatusBadge = (status: string) => {
        const normalizedStatus = status.toUpperCase();
        switch (normalizedStatus) {
            case 'PENDING':
                return <span className="px-3 py-1 rounded-full text-sm font-medium bg-yellow-100 text-yellow-800">Pending</span>;
            case 'APPROVED':
                return <span className="px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-800">Approved</span>;
            case 'REJECTED':
                return <span className="px-3 py-1 rounded-full text-sm font-medium bg-red-100 text-red-800">Rejected</span>;
            default:
                return null;
        }
    };

    const isPending = (status: string) => status.toUpperCase() === 'PENDING';

    if (loading) {
        return (
            <AnimatedPage>
                <div className="max-w-4xl mx-auto py-8 px-4">
                    <div className="flex justify-center items-center h-64">
                        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                    </div>
                </div>
            </AnimatedPage>
        );
    }

    return (
        <AnimatedPage>
            <div className="max-w-4xl mx-auto py-8 px-4">
                <div className="bg-white rounded-2xl shadow-xl overflow-hidden">
                    <div className="bg-gradient-to-r from-indigo-600 to-blue-600 px-6 py-8">
                        <h1 className="text-3xl font-bold text-white">
                            {isOwner ? 'Refund Requests' : 'My Refund History'}
                        </h1>
                        <p className="text-indigo-100 mt-2">
                            {isOwner
                                ? 'Review and manage refund requests from clients'
                                : 'Track the status of your refund requests'}
                        </p>
                    </div>

                    <div className="p-6">
                        {refunds.length === 0 ? (
                            <div className="text-center py-12">
                                <div className="text-6xl mb-4">📭</div>
                                <h3 className="text-xl font-semibold text-gray-700">No refund requests</h3>
                                <p className="text-gray-500 mt-2">
                                    {isOwner
                                        ? "You don't have any pending refund requests"
                                        : "You haven't made any refund requests yet"}
                                </p>
                            </div>
                        ) : (
                            <div className="space-y-4">
                                {refunds.map((refund) => (
                                    <div
                                        key={refund.id}
                                        className="border border-gray-200 rounded-xl p-5 hover:shadow-md transition-shadow"
                                    >
                                        <div className="flex justify-between items-start">
                                            <div className="flex-1">
                                                <div className="flex items-center gap-3 mb-2">
                                                    <span className="font-mono text-sm bg-gray-100 px-2 py-1 rounded">
                                                        {refund.ticket_cod}
                                                    </span>
                                                    {getStatusBadge(refund.status)}
                                                </div>

                                                {isOwner && (
                                                    <p className="text-sm text-gray-600 mb-1">
                                                        From: <span className="font-medium">{refund.requester_email}</span>
                                                    </p>
                                                )}

                                                <p className="text-gray-700 mt-2">
                                                    <span className="font-medium">Reason:</span> {refund.reason}
                                                </p>

                                                {refund.rejection_message && (
                                                    <p className="text-red-600 mt-2 text-sm">
                                                        <span className="font-medium">Rejection reason:</span> {refund.rejection_message}
                                                    </p>
                                                )}

                                                {refund.created_at && (
                                                    <p className="text-xs text-gray-400 mt-3">
                                                        Submitted: {new Date(refund.created_at).toLocaleString()}
                                                    </p>
                                                )}
                                            </div>

                                            {isOwner && isPending(refund.status) && (
                                                <div className="flex gap-2 ml-4">
                                                    <button
                                                        onClick={() => setConfirmAction({ id: refund.id, action: 'approve' })}
                                                        className="px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition-colors font-medium"
                                                    >
                                                        Approve
                                                    </button>
                                                    <button
                                                        onClick={() => setConfirmAction({ id: refund.id, action: 'reject' })}
                                                        className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors font-medium"
                                                    >
                                                        Reject
                                                    </button>
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* Approve Confirmation Modal */}
            <ConfirmModal
                isOpen={confirmAction?.action === 'approve'}
                title="Approve Refund"
                message="Are you sure you want to approve this refund request? The client will be notified and the ticket will be cancelled."
                onConfirm={() => confirmAction && handleApprove(confirmAction.id)}
                onCancel={() => setConfirmAction(null)}
            />

            {/* Reject Modal with reason input */}
            {confirmAction?.action === 'reject' && (
                <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
                    <div className="bg-white rounded-2xl p-6 max-w-md w-full mx-4 shadow-2xl">
                        <h3 className="text-xl font-bold text-gray-800 mb-4">Reject Refund</h3>
                        <p className="text-gray-600 mb-4">Please provide a reason for rejecting this refund:</p>
                        <textarea
                            value={rejectReason}
                            onChange={(e) => setRejectReason(e.target.value)}
                            className="w-full border border-gray-300 rounded-lg p-3 mb-4 focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                            rows={3}
                            placeholder="Enter rejection reason..."
                        />
                        <div className="flex gap-3 justify-end">
                            <button
                                onClick={() => {
                                    setConfirmAction(null);
                                    setRejectReason('');
                                }}
                                className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={() => handleReject(confirmAction.id, rejectReason)}
                                disabled={!rejectReason.trim()}
                                className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                Reject
                            </button>
                        </div>
                    </div>
                </div>
            )}

            <ErrorModal
                isOpen={!!error}
                title="Error"
                message={error || ''}
                onClose={() => setError(null)}
            />

            <SuccessModal
                isOpen={!!successMessage}
                title="Success"
                message={successMessage || ''}
                onClose={() => setSuccessMessage(null)}
            />
        </AnimatedPage>
    );
};
