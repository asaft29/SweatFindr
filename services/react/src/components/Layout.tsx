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
    <div className="min-h-screen flex flex-col bg-gray-50">
      <header className="bg-gradient-to-r from-indigo-600 to-blue-600 shadow-lg">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex-shrink-0">
              <h1 className="text-2xl font-bold text-white cursor-pointer hover:text-indigo-100 transition" onClick={() => navigate('/')}>
                SweatFindr
              </h1>
            </div>

            <nav className="flex items-center space-x-1 sm:space-x-2">
              <button onClick={() => navigate('/')} className="px-3 py-2 rounded-md text-sm font-medium text-white hover:bg-indigo-700 hover:bg-opacity-75 transition">
                Home
              </button>
              <button onClick={() => navigate('/events')} className="px-3 py-2 rounded-md text-sm font-medium text-white hover:bg-indigo-700 hover:bg-opacity-75 transition">
                Events
              </button>
              <button onClick={() => navigate('/packages')} className="px-3 py-2 rounded-md text-sm font-medium text-white hover:bg-indigo-700 hover:bg-opacity-75 transition">
                Packages
              </button>

              {isAuthenticated ? (
                <>
                  {user?.role === 'owner-event' && (
                    <button onClick={() => navigate('/my-events')} className="px-3 py-2 rounded-md text-sm font-medium text-white hover:bg-indigo-700 hover:bg-opacity-75 transition">
                      My Events
                    </button>
                  )}
                  {user?.role === 'client' && (
                    <button onClick={() => navigate('/my-tickets')} className="px-3 py-2 rounded-md text-sm font-medium text-white hover:bg-indigo-700 hover:bg-opacity-75 transition">
                      My Tickets
                    </button>
                  )}
                  <div className="flex items-center space-x-3 ml-4 pl-4 border-l border-indigo-400">
                    <span className="text-sm text-indigo-100">{user?.email}</span>
                    <button onClick={handleLogout} className="px-4 py-2 rounded-md text-sm font-medium text-indigo-600 bg-white hover:bg-indigo-50 transition">
                      Logout
                    </button>
                  </div>
                </>
              ) : (
                <>
                  <button onClick={() => navigate('/login')} className="px-3 py-2 rounded-md text-sm font-medium text-white hover:bg-indigo-700 hover:bg-opacity-75 transition">
                    Sign In
                  </button>
                  <button onClick={() => navigate('/register')} className="ml-2 px-4 py-2 rounded-md text-sm font-medium text-indigo-600 bg-white hover:bg-indigo-50 transition">
                    Sign Up
                  </button>
                </>
              )}
            </nav>
          </div>
        </div>
      </header>

      <main className="flex-grow">{children}</main>

      <footer className="bg-white border-t border-gray-200">
        <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
          <p className="text-center text-gray-500 text-sm">&copy; 2025 SweatFindr. All rights reserved.</p>
        </div>
      </footer>
    </div>
  );
};
