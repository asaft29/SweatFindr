import { useState, FormEvent } from 'react';
import { useNavigate } from 'react-router-dom';
import { apiClient } from '../lib/api';

export const ResendVerificationForm = () => {
  const navigate = useNavigate();
  const [email, setEmail] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string>('');

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);

    if (!email) {
      setError('Email is required');
      setIsLoading(false);
      return;
    }

    try {
      const response = await apiClient.getGateway().post('/api/email/resend-by-email', { email });
      if (response.data.success && response.data.user_id > 0) {
        sessionStorage.setItem('verify_email', email);
        sessionStorage.setItem('verify_user_id', response.data.user_id.toString());
        navigate(`/verify-email?userId=${response.data.user_id}&email=${encodeURIComponent(email)}`);
      } else {
        setError('If your email is registered and unverified, you will receive a verification code.');
      }
    } catch (err) {
      setError('Failed to send verification code. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8 bg-white p-8 rounded-xl shadow-2xl">
        <div>
          <h1 className="text-center text-3xl font-extrabold text-gray-900">Resend Verification Code</h1>
          <p className="mt-2 text-center text-sm text-gray-600">
            Enter your email address and we'll send you a new verification code
          </p>
        </div>

        {error && (
          <div className="bg-yellow-50 border border-yellow-200 text-yellow-800 px-4 py-3 rounded-lg" role="alert">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="mt-8 space-y-6">
          <div>
            <label htmlFor="email" className="block text-sm font-medium text-gray-700 mb-1">
              Email Address
            </label>
            <input
              id="email"
              type="email"
              className="appearance-none rounded-lg relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm transition"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="Enter your email"
              disabled={isLoading}
              autoComplete="email"
            />
          </div>

          <button
            type="submit"
            className="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-lg text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition"
            disabled={isLoading}
          >
            {isLoading ? 'Sending...' : 'Send Verification Code'}
          </button>
        </form>

        <div className="text-center">
          <a
            href="/login"
            className="text-sm font-medium text-indigo-600 hover:text-indigo-500 transition"
          >
            Back to Sign In
          </a>
        </div>
      </div>
    </div>
  );
};
