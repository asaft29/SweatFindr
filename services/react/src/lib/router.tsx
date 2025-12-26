import { createBrowserRouter } from 'react-router-dom';
import { Layout } from '../components/Layout';
import { ProtectedRoute } from '../components/ProtectedRoute';
import { LoginForm } from '../components/LoginForm';
import { RegisterForm } from '../components/RegisterForm';
import { VerifyEmailForm } from '../components/VerifyEmailForm';
import { HomePage } from '../pages/HomePage';
import { UnauthorizedPage } from '../pages/UnauthorizedPage';
import { EventsPage } from '../pages/EventsPage';
import { EventPackagesPage } from '../pages/EventPackagesPage';

export const router = createBrowserRouter([
  {
    path: '/',
    element: (
      <Layout>
        <HomePage />
      </Layout>
    ),
  },
  {
    path: '/login',
    element: <LoginForm />,
  },
  {
    path: '/register',
    element: <RegisterForm />,
  },
  {
    path: '/verify-email',
    element: <VerifyEmailForm />,
  },
  {
    path: '/unauthorized',
    element: (
      <Layout>
        <UnauthorizedPage />
      </Layout>
    ),
  },
  {
    path: '/events',
    element: (
      <Layout>
        <EventsPage />
      </Layout>
    ),
  },
  {
    path: '/packages',
    element: (
      <Layout>
        <EventPackagesPage />
      </Layout>
    ),
  },
  {
    path: '/my-events',
    element: (
      <Layout>
        <ProtectedRoute allowedRoles={['owner-event'] as any}>
          <div>
            <h1>My Events</h1>
            <p>Manage your events (Event Owner only)</p>
          </div>
        </ProtectedRoute>
      </Layout>
    ),
  },
  {
    path: '/my-tickets',
    element: (
      <Layout>
        <ProtectedRoute allowedRoles={['client'] as any}>
          <div>
            <h1>My Tickets</h1>
            <p>View your purchased tickets (Client only)</p>
          </div>
        </ProtectedRoute>
      </Layout>
    ),
  },
]);
