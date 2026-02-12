import { useAuthStore } from '../lib/useAuthStore';
import { UserRole } from '../lib/types';

export const HomePage = () => {
  const { user, isAuthenticated } = useAuthStore();

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-5xl font-extrabold text-gray-900 mb-6">
          Welcome to CargoTicket
        </h1>

        {isAuthenticated && user ? (
          <div className="space-y-6">
            <p className="text-xl text-gray-700">
              Hello, <span className="font-semibold text-indigo-600">{user.email}</span>!
            </p>

            {user.role === UserRole.CLIENT && (
              <div className="bg-white rounded-xl shadow-lg p-8">
                <h2 className="text-3xl font-bold text-gray-900 mb-4">Client Dashboard</h2>
                <p className="text-lg text-gray-700">Browse events, purchase tickets, and manage your bookings.</p>
              </div>
            )}

            {user.role === UserRole.OWNER_EVENT && (
              <div className="bg-white rounded-xl shadow-lg p-8">
                <h2 className="text-3xl font-bold text-gray-900 mb-4">Event Owner Dashboard</h2>
                <p className="text-lg text-gray-700">Manage your events, view ticket sales, and track attendees.</p>
              </div>
            )}
          </div>
        ) : (
          <div className="space-y-6">
            <p className="text-xl text-gray-700">
              Please sign in to access all features.
            </p>
            <div className="bg-white rounded-xl shadow-lg p-8">
              <h2 className="text-3xl font-bold text-gray-900 mb-4">Guest Access</h2>
              <p className="text-lg text-gray-700">Browse public events and packages without signing in.</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
