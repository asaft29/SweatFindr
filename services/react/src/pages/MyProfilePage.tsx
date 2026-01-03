import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { clientService } from "../lib/clientService";
import { useAuthStore } from "../lib/useAuthStore";
import type { Client } from "../lib/types";
import { FaInstagram, FaFacebook, FaTwitter, FaLinkedin, FaGithub } from "react-icons/fa";

export function MyProfilePage() {
  const navigate = useNavigate();
  const { logout } = useAuthStore();
  const [profile, setProfile] = useState<Client | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [deleteConfirmEmail, setDeleteConfirmEmail] = useState("");
  const [deleteLoading, setDeleteLoading] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    prenume: "",
    nume: "",
    public_info: false,
    instagram: "",
    facebook: "",
    twitter: "",
    linkedin: "",
    github: "",
  });

  const loadProfile = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await clientService.getMyProfile();
      setProfile(data);
      setFormData({
        prenume: data.prenume || "",
        nume: data.nume || "",
        public_info: data.public_info || false,
        instagram: data.social_media?.instagram || "",
        facebook: data.social_media?.facebook || "",
        twitter: data.social_media?.twitter || "",
        linkedin: data.social_media?.linkedin || "",
        github: data.social_media?.github || "",
      });
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || "Failed to load profile");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      setLoading(true);
      setError(null);
      const updated = await clientService.updateMyProfile({
        prenume: formData.prenume || undefined,
        nume: formData.nume || undefined,
        public_info: formData.public_info,
        social_media: {
          instagram: formData.instagram || undefined,
          facebook: formData.facebook || undefined,
          twitter: formData.twitter || undefined,
          linkedin: formData.linkedin || undefined,
          github: formData.github || undefined,
        },
      });
      setProfile(updated);
      setEditing(false);
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || "Failed to update profile");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteAccount = async () => {
    if (deleteConfirmEmail !== profile?.email) {
      setDeleteError("Email does not match");
      return;
    }

    try {
      setDeleteLoading(true);
      setDeleteError(null);
      await clientService.deleteMyAccount();
      logout();
      navigate("/login");
    } catch (err: any) {
      if (err.response?.status === 403) {
        setDeleteError("You still have active tickets!");
      } else {
        const errorMessage = err.response?.data?.error || err.message || "Failed to delete account";
        setDeleteError(errorMessage);
      }
    } finally {
      setDeleteLoading(false);
    }
  };

  const closeDeleteModal = () => {
    setShowDeleteModal(false);
    setDeleteConfirmEmail("");
    setDeleteError(null);
  };

  useEffect(() => {
    loadProfile();
  }, []);

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-3xl mx-auto">
        {loading && !profile && (
          <div className="flex items-center justify-center py-12">
            <div className="inline-block animate-spin rounded-full h-12 w-12 border-t-4 border-b-4 border-indigo-600"></div>
          </div>
        )}

        {error && (
          <div className="mb-4">
            <div className="bg-red-50 border border-red-200 text-red-800 px-6 py-4 rounded-lg">
              {error}
            </div>
          </div>
        )}

        {profile && !editing && (
          <div className="bg-white rounded-xl shadow-lg p-8">
            <div className="space-y-4">
              <div>
                <p className="text-sm text-gray-500">Email</p>
                <p className="text-lg font-medium text-gray-900">{profile.email}</p>
              </div>
              <div>
                <p className="text-sm text-gray-500">First Name</p>
                <p className="text-lg font-medium text-gray-900">{profile.prenume}</p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Last Name</p>
                <p className="text-lg font-medium text-gray-900">{profile.nume}</p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Profile Visibility</p>
                <p className="text-lg font-medium text-gray-900">
                  {profile.public_info ? "Public" : "Private"}
                </p>
              </div>
              {(profile.social_media?.instagram || profile.social_media?.facebook || profile.social_media?.twitter || profile.social_media?.linkedin || profile.social_media?.github) && (
                <div>
                  <p className="text-sm text-gray-500 mb-2">Social Media</p>
                  <div className="flex gap-4">
                    {profile.social_media.instagram && (
                      <a
                        href={profile.social_media.instagram.startsWith('http') ? profile.social_media.instagram : `https://instagram.com/${profile.social_media.instagram.replace('@', '')}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-pink-500 hover:text-pink-600 hover:scale-110 transition-all"
                        title="Instagram"
                      >
                        <FaInstagram size={32} />
                      </a>
                    )}
                    {profile.social_media.facebook && (
                      <a
                        href={profile.social_media.facebook.startsWith('http') ? profile.social_media.facebook : `https://facebook.com/${profile.social_media.facebook}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-600 hover:text-blue-700 hover:scale-110 transition-all"
                        title="Facebook"
                      >
                        <FaFacebook size={32} />
                      </a>
                    )}
                    {profile.social_media.twitter && (
                      <a
                        href={profile.social_media.twitter.startsWith('http') ? profile.social_media.twitter : `https://twitter.com/${profile.social_media.twitter.replace('@', '')}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-sky-500 hover:text-sky-600 hover:scale-110 transition-all"
                        title="Twitter / X"
                      >
                        <FaTwitter size={32} />
                      </a>
                    )}
                    {profile.social_media.linkedin && (
                      <a
                        href={profile.social_media.linkedin.startsWith('http') ? profile.social_media.linkedin : `https://linkedin.com/in/${profile.social_media.linkedin}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-700 hover:text-blue-800 hover:scale-110 transition-all"
                        title="LinkedIn"
                      >
                        <FaLinkedin size={32} />
                      </a>
                    )}
                    {profile.social_media.github && (
                      <a
                        href={profile.social_media.github.startsWith('http') ? profile.social_media.github : `https://github.com/${profile.social_media.github}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-gray-800 hover:text-gray-900 hover:scale-110 transition-all"
                        title="GitHub"
                      >
                        <FaGithub size={32} />
                      </a>
                    )}
                  </div>
                </div>
              )}
            </div>
            <div className="flex gap-3 mt-6">
              <button
                onClick={() => setEditing(true)}
                className="px-6 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition"
              >
                Edit Profile
              </button>
              <button
                onClick={() => setShowDeleteModal(true)}
                className="px-6 py-2.5 text-sm font-bold text-white bg-red-600 hover:bg-red-700 rounded-lg transition"
              >
                Delete Account
              </button>
            </div>
          </div>
        )}

        {profile && editing && (
          <form onSubmit={handleSubmit} className="bg-white rounded-xl shadow-lg p-8">
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Email (read-only)
                </label>
                <input
                  type="text"
                  value={profile.email}
                  disabled
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg bg-gray-100 text-gray-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  First Name *
                </label>
                <input
                  type="text"
                  value={formData.prenume}
                  onChange={(e) => setFormData({ ...formData, prenume: e.target.value })}
                  required
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Last Name *
                </label>
                <input
                  type="text"
                  value={formData.nume}
                  onChange={(e) => setFormData({ ...formData, nume: e.target.value })}
                  required
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                />
              </div>
              <div className="flex items-center gap-3">
                <input
                  type="checkbox"
                  id="public_info"
                  checked={formData.public_info}
                  onChange={(e) => setFormData({ ...formData, public_info: e.target.checked })}
                  className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                />
                <label htmlFor="public_info" className="text-sm font-medium text-gray-700">
                  Make profile public
                </label>
              </div>
              <div className="border-t pt-4 mt-2">
                <p className="text-sm font-medium text-gray-700 mb-3">Social Media Links</p>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Instagram
                    </label>
                    <input
                      type="text"
                      value={formData.instagram}
                      onChange={(e) => setFormData({ ...formData, instagram: e.target.value })}
                      placeholder="@username or profile URL"
                      className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Facebook
                    </label>
                    <input
                      type="text"
                      value={formData.facebook}
                      onChange={(e) => setFormData({ ...formData, facebook: e.target.value })}
                      placeholder="Profile URL"
                      className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Twitter / X
                    </label>
                    <input
                      type="text"
                      value={formData.twitter}
                      onChange={(e) => setFormData({ ...formData, twitter: e.target.value })}
                      placeholder="@username or profile URL"
                      className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      LinkedIn
                    </label>
                    <input
                      type="text"
                      value={formData.linkedin}
                      onChange={(e) => setFormData({ ...formData, linkedin: e.target.value })}
                      placeholder="Profile URL"
                      className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      GitHub
                    </label>
                    <input
                      type="text"
                      value={formData.github}
                      onChange={(e) => setFormData({ ...formData, github: e.target.value })}
                      placeholder="Username or profile URL"
                      className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                    />
                  </div>
                </div>
              </div>
            </div>
            <div className="flex gap-3 mt-6">
              <button
                type="submit"
                disabled={loading}
                className="px-6 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition disabled:bg-gray-400"
              >
                {loading ? "Saving..." : "Save Changes"}
              </button>
              <button
                type="button"
                onClick={() => {
                  setEditing(false);
                  setFormData({
                    prenume: profile.prenume || "",
                    nume: profile.nume || "",
                    public_info: profile.public_info || false,
                    instagram: profile.social_media?.instagram || "",
                    facebook: profile.social_media?.facebook || "",
                    twitter: profile.social_media?.twitter || "",
                    linkedin: profile.social_media?.linkedin || "",
                    github: profile.social_media?.github || "",
                  });
                }}
                className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition"
              >
                Cancel
              </button>
            </div>
          </form>
        )}

        {showDeleteModal && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
            <div className="bg-white rounded-xl shadow-2xl max-w-md w-full p-6">
              <h2 className="text-xl font-bold text-red-600 mb-4">Delete Account</h2>
              <p className="text-gray-600 mb-4">
                This action is permanent and cannot be undone. All your data will be deleted.
              </p>
              <p className="text-gray-600 mb-4">
                To confirm, please type your email: <strong>{profile?.email}</strong>
              </p>

              {deleteError && (
                <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
                  {deleteError}
                </div>
              )}

              <input
                type="email"
                value={deleteConfirmEmail}
                onChange={(e) => setDeleteConfirmEmail(e.target.value)}
                placeholder="Enter your email to confirm"
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-red-500 mb-4"
                disabled={deleteLoading}
              />

              <div className="flex gap-3 justify-end">
                <button
                  onClick={closeDeleteModal}
                  disabled={deleteLoading}
                  className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition disabled:opacity-50"
                >
                  Cancel
                </button>
                <button
                  onClick={handleDeleteAccount}
                  disabled={deleteLoading || deleteConfirmEmail !== profile?.email}
                  className="px-4 py-2 text-sm font-bold text-white bg-red-600 hover:bg-red-700 rounded-lg transition disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {deleteLoading ? "Deleting..." : "Delete My Account"}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
