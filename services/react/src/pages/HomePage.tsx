import { useAuthStore } from '../lib/useAuthStore';
import { UserRole } from '../lib/types';

export const HomePage = () => {
  const { user, isAuthenticated } = useAuthStore();

  return (
    <div style={{ padding: '2rem' }}>
      <h1 style={{ fontSize: '2.5rem', marginBottom: '1rem' }}>
        Welcome to Event Platform
      </h1>

      {isAuthenticated && user ? (
        <div>
          <p style={{ fontSize: '1.2rem', color: '#666', marginBottom: '2rem' }}>
            Hello, {user.email}!
          </p>

          {user.role === UserRole.CLIENT && (
            <div>
              <h2>Client Dashboard</h2>
              <p>Browse events, purchase tickets, and manage your bookings.</p>
            </div>
          )}

          {user.role === UserRole.OWNER_EVENT && (
            <div>
              <h2>Event Owner Dashboard</h2>
              <p>Manage your events, view ticket sales, and track attendees.</p>
            </div>
          )}
        </div>
      ) : (
        <div>
          <p style={{ fontSize: '1.2rem', color: '#666', marginBottom: '2rem' }}>
            Please sign in to access all features.
          </p>
          <h2>Guest Access</h2>
          <p>Browse public events and packages without signing in.</p>
        </div>
      )}
    </div>
  );
};
