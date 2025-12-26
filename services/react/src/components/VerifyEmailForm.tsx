import { useState, FormEvent } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { authService } from '../lib/authService';

export const VerifyEmailForm = () => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const userId = searchParams.get('userId');
  const email = searchParams.get('email');

  const [verificationCode, setVerificationCode] = useState('');
  const [error, setError] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [isResending, setIsResending] = useState(false);

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');

    if (!verificationCode) {
      setError('Please enter the verification code');
      return;
    }

    if (!userId) {
      setError('Invalid verification link');
      return;
    }

    setIsLoading(true);
    try {
      const response = await authService.verifyEmail({
        user_id: parseInt(userId),
        verification_code: verificationCode,
      });

      if (response.success) {
        navigate('/login?verified=true');
      } else {
        setError(response.message || 'Verification failed');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification failed. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleResend = async () => {
    if (!userId || !email) {
      setError('Invalid verification link');
      return;
    }

    setIsResending(true);
    setError('');

    try {
      const response = await authService.resendVerification({
        user_id: parseInt(userId),
        email,
      });

      if (response.success) {
        setError('');
      } else {
        setError(response.message || 'Failed to resend code');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to resend code');
    } finally {
      setIsResending(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8 bg-white p-8 rounded-xl shadow-2xl">
        <div>
          <h1 className="text-center text-3xl font-extrabold text-gray-900">Verify Your Email</h1>
          <p className="mt-2 text-center text-sm text-gray-600">
            We sent a verification code to {email || 'your email'}
          </p>
        </div>

        {error && (
          <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg" role="alert">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="mt-8 space-y-6">
          <div>
            <label htmlFor="code" className="block text-sm font-medium text-gray-700 mb-1">
              Verification Code
            </label>
            <input
              id="code"
              type="text"
              className="appearance-none rounded-lg relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm transition"
              value={verificationCode}
              onChange={(e) => setVerificationCode(e.target.value)}
              placeholder="Enter 6-digit code"
              disabled={isLoading}
              maxLength={6}
              autoComplete="off"
            />
          </div>

          <button
            type="submit"
            className="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-lg text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition"
            disabled={isLoading}
          >
            {isLoading ? 'Verifying...' : 'Verify Email'}
          </button>
        </form>

        <div className="text-center">
          <p className="text-sm text-gray-600">
            Didn't receive the code?{' '}
            <button
              onClick={handleResend}
              disabled={isResending}
              className="font-medium text-indigo-600 hover:text-indigo-500 transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isResending ? 'Sending...' : 'Resend'}
            </button>
          </p>
        </div>
      </div>
    </div>
  );
};
