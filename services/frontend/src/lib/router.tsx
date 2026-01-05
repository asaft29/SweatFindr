import { createBrowserRouter } from 'react-router-dom';
import { Layout } from '../components/Layout';
import { AnimatedPage } from '../components/AnimatedPage';
import { ProtectedRoute } from '../components/ProtectedRoute';
import { LoginForm } from '../components/LoginForm';
import { RegisterForm } from '../components/RegisterForm';
import { VerifyEmailForm } from '../components/VerifyEmailForm';
import { ForgotPasswordForm } from '../components/ForgotPasswordForm';
import { ResetPasswordForm } from '../components/ResetPasswordForm';
import { HomePage } from '../pages/HomePage';
import { UnauthorizedPage } from '../pages/UnauthorizedPage';
import { EventsPage } from '../pages/EventsPage';
import { EventPackagesPage } from '../pages/EventPackagesPage';
import { MyTicketsPage } from '../pages/MyTicketsPage';
import { MyProfilePage } from '../pages/MyProfilePage';
import { MyEventsPage } from '../pages/MyEventsPage';
import { InboxPage } from '../pages/InboxPage';

export const router = createBrowserRouter([
  {
    path: '/',
    element: (
      <Layout>
        <AnimatedPage>
          <HomePage />
        </AnimatedPage>
      </Layout>
    ),
  },
  {
    path: '/login',
    element: (
      <AnimatedPage>
        <LoginForm />
      </AnimatedPage>
    ),
  },
  {
    path: '/register',
    element: (
      <AnimatedPage>
        <RegisterForm />
      </AnimatedPage>
    ),
  },
  {
    path: '/verify-email',
    element: (
      <AnimatedPage>
        <VerifyEmailForm />
      </AnimatedPage>
    ),
  },
  {
    path: '/forgot-password',
    element: (
      <AnimatedPage>
        <ForgotPasswordForm />
      </AnimatedPage>
    ),
  },
  {
    path: '/reset-password',
    element: (
      <AnimatedPage>
        <ResetPasswordForm />
      </AnimatedPage>
    ),
  },
  {
    path: '/unauthorized',
    element: (
      <Layout>
        <AnimatedPage>
          <UnauthorizedPage />
        </AnimatedPage>
      </Layout>
    ),
  },
  {
    path: '/events',
    element: (
      <Layout>
        <AnimatedPage>
          <EventsPage />
        </AnimatedPage>
      </Layout>
    ),
  },
  {
    path: '/packages',
    element: (
      <Layout>
        <AnimatedPage>
          <EventPackagesPage />
        </AnimatedPage>
      </Layout>
    ),
  },
  {
    path: '/my-events',
    element: (
      <Layout>
        <AnimatedPage>
          <ProtectedRoute allowedRoles={['owner-event'] as any}>
            <MyEventsPage />
          </ProtectedRoute>
        </AnimatedPage>
      </Layout>
    ),
  },
  {
    path: '/my-tickets',
    element: (
      <Layout>
        <AnimatedPage>
          <ProtectedRoute allowedRoles={['client'] as any}>
            <MyTicketsPage />
          </ProtectedRoute>
        </AnimatedPage>
      </Layout>
    ),
  },
  {
    path: '/my-profile',
    element: (
      <Layout>
        <AnimatedPage>
          <ProtectedRoute allowedRoles={['client'] as any}>
            <MyProfilePage />
          </ProtectedRoute>
        </AnimatedPage>
      </Layout>
    ),
  },
  {
    path: '/inbox',
    element: (
      <Layout>
        <ProtectedRoute allowedRoles={['client', 'owner-event'] as any}>
          <InboxPage />
        </ProtectedRoute>
      </Layout>
    ),
  },
]);
