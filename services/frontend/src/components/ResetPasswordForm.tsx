import { useState, FormEvent, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { apiClient } from '../lib/api';
import { useAuthStore } from '../lib/useAuthStore';

export const ResetPasswordForm = () => {
  const navigate = useNavigate();
  const { logout, isAuthenticated } = useAuthStore();
  const [email, setEmail] = useState('');
  const [resetCode, setResetCode] = useState('');
  const [resetToken, setResetToken] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState(false);
  const [step, setStep] = useState<'code' | 'password'>('code');

  useEffect(() => {
    const storedEmail = sessionStorage.getItem('reset_email');
    if (storedEmail) {
      setEmail(storedEmail);
    } else {
      navigate('/forgot-password');
    }
  }, [navigate]);

  const handleVerifyCode = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);

    if (!resetCode) {
      setError('Reset code is required');
      setIsLoading(false);
      return;
    }

    try {
      const response = await apiClient.getGateway().post('/api/email/verify-reset-code', {
        email,
        reset_code: resetCode,
      });

      if (response.data.success && response.data.reset_token) {
        setResetToken(response.data.reset_token);
        setStep('password');
      } else {
        setError(response.data.message || 'Invalid or expired reset code');
      }
    } catch (err) {
      if (err instanceof Error && err.name === 'RateLimitError') {
        setError(err.message);
      } else {
        setError('Invalid or expired reset code');
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleResetPassword = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);

    if (!newPassword || !confirmPassword) {
      setError('Both password fields are required');
      setIsLoading(false);
      return;
    }

    if (newPassword !== confirmPassword) {
      setError('Passwords do not match');
      setIsLoading(false);
      return;
    }

    if (newPassword.length < 8) {
      setError('Password must be at least 8 characters');
      setIsLoading(false);
      return;
    }

    try {
      const response = await apiClient.getGateway().post('/api/email/reset-password', {
        email,
        new_password: newPassword,
        reset_token: resetToken,
      });

      if (response.data.success) {
        setSuccess(true);
        sessionStorage.removeItem('reset_email');

        if (isAuthenticated) {
          await logout();
        }

        setTimeout(() => {
          navigate('/login');
        }, 2000);
      } else {
        setError(response.data.message || 'Failed to reset password');
      }
    } catch (err) {
      if (err instanceof Error && err.name === 'RateLimitError') {
        setError(err.message);
      } else {
        setError('Failed to reset password. Please try again.');
      }
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8 bg-white p-8 rounded-xl shadow-2xl">
        <div>
          <h1 className="text-center text-3xl font-extrabold text-gray-900">Reset Password</h1>
          <p className="mt-2 text-center text-sm text-gray-600">
            {step === 'code'
              ? 'Enter the code sent to your email'
              : 'Enter your new password'}
          </p>
        </div>

        {error && (
          <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg" role="alert">
            {error}
          </div>
        )}

        {success && (
          <div className="bg-green-50 border border-green-200 text-green-800 px-4 py-3 rounded-lg" role="alert">
            Password reset successful! Redirecting to login...
          </div>
        )}

        {step === 'code' ? (
          <form onSubmit={handleVerifyCode} className="mt-8 space-y-6">
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-gray-700 mb-1">
                Email Address
              </label>
              <input
                id="email"
                type="email"
                className="appearance-none rounded-lg relative block w-full px-3 py-2 border border-gray-300 bg-gray-100 text-gray-900 sm:text-sm"
                value={email}
                disabled
              />
            </div>

            <div>
              <label htmlFor="resetCode" className="block text-sm font-medium text-gray-700 mb-1">
                Reset Code
              </label>
              <input
                id="resetCode"
                type="text"
                className="appearance-none rounded-lg relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm transition text-center text-2xl tracking-widest"
                value={resetCode}
                onChange={(e) => setResetCode(e.target.value)}
                placeholder="000000"
                disabled={isLoading}
                maxLength={6}
                autoComplete="one-time-code"
              />
            </div>

            <button
              type="submit"
              className="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-lg text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition"
              disabled={isLoading}
            >
              {isLoading ? 'Verifying...' : 'Verify Code'}
            </button>
          </form>
        ) : (
          <form onSubmit={handleResetPassword} className="mt-8 space-y-6">
            <div>
              <label htmlFor="newPassword" className="block text-sm font-medium text-gray-700 mb-1">
                New Password
              </label>
              <input
                id="newPassword"
                type="password"
                className="appearance-none rounded-lg relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm transition"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder="Enter new password"
                disabled={isLoading || success}
                autoComplete="new-password"
              />
            </div>

            <div>
              <label htmlFor="confirmPassword" className="block text-sm font-medium text-gray-700 mb-1">
                Confirm Password
              </label>
              <input
                id="confirmPassword"
                type="password"
                className="appearance-none rounded-lg relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm transition"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="Confirm new password"
                disabled={isLoading || success}
                autoComplete="new-password"
              />
            </div>

            <button
              type="submit"
              className="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-lg text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed transition"
              disabled={isLoading || success}
            >
              {isLoading ? 'Resetting...' : 'Reset Password'}
            </button>
          </form>
        )}

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
