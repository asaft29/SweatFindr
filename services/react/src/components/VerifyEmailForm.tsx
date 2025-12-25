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
    <div className="auth-container">
      <div className="auth-card">
        <h1 className="auth-title">Verify Your Email</h1>
        <p className="auth-subtitle">
          We sent a verification code to {email || 'your email'}
        </p>

        {error && (
          <div className="auth-error" role="alert">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="auth-form">
          <div className="form-group">
            <label htmlFor="code" className="form-label">
              Verification Code
            </label>
            <input
              id="code"
              type="text"
              className="form-input"
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
            className="auth-button"
            disabled={isLoading}
          >
            {isLoading ? 'Verifying...' : 'Verify Email'}
          </button>
        </form>

        <div className="auth-footer">
          <p>
            Didn't receive the code?{' '}
            <button
              onClick={handleResend}
              disabled={isResending}
              className="auth-link-button"
            >
              {isResending ? 'Sending...' : 'Resend'}
            </button>
          </p>
        </div>
      </div>
    </div>
  );
};
