import { ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '../lib/useAuthStore';

interface LayoutProps {
  children: ReactNode;
}

export const Layout = ({ children }: LayoutProps) => {
  const navigate = useNavigate();
  const { isAuthenticated, user, logout } = useAuthStore();

  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  return (
    <div className="layout">
      <header className="header">
        <div className="header-content">
          <div className="header-left">
            <h1 className="logo" onClick={() => navigate('/')}>
              Event Platform
            </h1>
          </div>

          <nav className="nav">
            {isAuthenticated ? (
              <>
                <button onClick={() => navigate('/')} className="nav-link">
                  Home
                </button>
                <button onClick={() => navigate('/events')} className="nav-link">
                  Events
                </button>
                {user?.role === 'owner-event' && (
                  <button onClick={() => navigate('/my-events')} className="nav-link">
                    My Events
                  </button>
                )}
                {user?.role === 'client' && (
                  <button onClick={() => navigate('/my-tickets')} className="nav-link">
                    My Tickets
                  </button>
                )}
                <div className="user-menu">
                  <span className="user-email">{user?.email}</span>
                  <button onClick={handleLogout} className="logout-button">
                    Logout
                  </button>
                </div>
              </>
            ) : (
              <>
                <button onClick={() => navigate('/login')} className="nav-link">
                  Sign In
                </button>
                <button onClick={() => navigate('/register')} className="register-button">
                  Sign Up
                </button>
              </>
            )}
          </nav>
        </div>
      </header>

      <main className="main-content">{children}</main>

      <footer className="footer">
        <p>&copy; 2025 Event Platform. All rights reserved.</p>
      </footer>
    </div>
  );
};
