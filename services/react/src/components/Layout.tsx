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
    <div className="min-h-screen flex flex-col bg-gradient-to-br from-blue-50 to-indigo-100">
      <header className="relative z-10 bg-gradient-to-r from-indigo-600 to-blue-600 shadow-md">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-24 py-4">
            <div className="flex-shrink-0">
              <h1 className="text-4xl font-black text-white cursor-pointer hover:text-indigo-200 transition-all duration-300 drop-shadow-2xl tracking-tight" onClick={() => navigate('/')}>
                SweatFindr
              </h1>
            </div>

            <nav className="flex items-center space-x-2 sm:space-x-3">
              <button onClick={() => navigate('/')} className="px-4 py-2.5 rounded-lg text-base font-semibold text-white hover:bg-white/20 backdrop-blur-sm transition-all duration-200 shadow-lg hover:shadow-xl">
                Home
              </button>
              <button onClick={() => navigate('/events')} className="px-4 py-2.5 rounded-lg text-base font-semibold text-white hover:bg-white/20 backdrop-blur-sm transition-all duration-200 shadow-lg hover:shadow-xl">
                Events
              </button>
              <button onClick={() => navigate('/packages')} className="px-4 py-2.5 rounded-lg text-base font-semibold text-white hover:bg-white/20 backdrop-blur-sm transition-all duration-200 shadow-lg hover:shadow-xl">
                Packages
              </button>

              {isAuthenticated ? (
                <>
                  {user?.role === 'owner-event' && (
                    <button onClick={() => navigate('/my-events')} className="px-4 py-2.5 rounded-lg text-base font-semibold text-white hover:bg-white/20 backdrop-blur-sm transition-all duration-200 shadow-lg hover:shadow-xl">
                      Tickets
                    </button>
                  )}
                  {user?.role === 'client' && (
                    <>
                      <button onClick={() => navigate('/my-tickets')} className="px-4 py-2.5 rounded-lg text-base font-semibold text-white hover:bg-white/20 backdrop-blur-sm transition-all duration-200 shadow-lg hover:shadow-xl">
                        Tickets
                      </button>
                      <button onClick={() => navigate('/my-profile')} className="px-4 py-2.5 rounded-lg text-base font-semibold text-white hover:bg-white/20 backdrop-blur-sm transition-all duration-200 shadow-lg hover:shadow-xl">
                        Profile
                      </button>
                    </>
                  )}
                  <div className="flex items-center space-x-3 ml-4 pl-4 border-l-2 border-white/30">
                    <span className="text-sm font-medium text-white drop-shadow-md">{user?.email}</span>
                    <button onClick={handleLogout} className="px-5 py-2.5 rounded-lg text-base font-bold text-indigo-900 bg-white hover:bg-indigo-50 transition-all duration-200 shadow-lg hover:shadow-xl">
                      Logout
                    </button>
                  </div>
                </>
              ) : (
                <>
                  <button onClick={() => navigate('/login')} className="px-4 py-2.5 rounded-lg text-base font-semibold text-white hover:bg-white/20 backdrop-blur-sm transition-all duration-200 shadow-lg hover:shadow-xl">
                    Sign In
                  </button>
                  <button onClick={() => navigate('/register')} className="ml-2 px-5 py-2.5 rounded-lg text-base font-bold text-indigo-900 bg-white hover:bg-indigo-50 transition-all duration-200 shadow-lg hover:shadow-xl">
                    Sign Up
                  </button>
                </>
              )}
            </nav>
          </div>
        </div>
      </header>

      <main className="flex-grow bg-gradient-to-br from-blue-50 to-indigo-100">{children}</main>

      <footer className="bg-white border-t border-gray-200">
        <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
          <p className="text-center text-gray-500 text-sm">&copy; 2025 SweatFindr. All rights reserved.</p>
        </div>
      </footer>
    </div>
  );
};
