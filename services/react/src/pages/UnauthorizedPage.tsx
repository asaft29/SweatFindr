import { useNavigate } from 'react-router-dom';

export const UnauthorizedPage = () => {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="text-center">
        <h1 className="text-8xl font-extrabold text-indigo-600 mb-4">403</h1>
        <h2 className="text-3xl font-bold text-gray-900 mb-4">
          Access Denied
        </h2>
        <p className="text-lg text-gray-600 mb-8">
          You don't have permission to access this page.
        </p>
        <button
          onClick={() => navigate('/')}
          className="px-6 py-3 border border-transparent text-base font-medium rounded-lg text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition"
        >
          Go Home
        </button>
      </div>
    </div>
  );
};
