import { useEffect, useState } from "react";
import { clientService } from "../lib/clientService";
import type { Client } from "../lib/types";

export function MyProfilePage() {
  const [profile, setProfile] = useState<Client | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [formData, setFormData] = useState({
    prenume: "",
    nume: "",
    public_info: "",
    instagram: "",
    facebook: "",
    twitter: "",
  });

  const loadProfile = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await clientService.getMyProfile();
      setProfile(data);
      setFormData({
        prenume: data.prenume,
        nume: data.nume,
        public_info: data.public_info || "",
        instagram: data.social_media?.instagram || "",
        facebook: data.social_media?.facebook || "",
        twitter: data.social_media?.twitter || "",
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
        prenume: formData.prenume,
        nume: formData.nume,
        public_info: formData.public_info,
        social_media: {
          instagram: formData.instagram,
          facebook: formData.facebook,
          twitter: formData.twitter,
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

  useEffect(() => {
    loadProfile();
  }, []);

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-3xl mx-auto">
        <h1 className="text-4xl font-extrabold text-gray-900 mb-8">My Profile</h1>

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
              {profile.public_info && (
                <div>
                  <p className="text-sm text-gray-500">Public Info</p>
                  <p className="text-lg text-gray-900">{profile.public_info}</p>
                </div>
              )}
              {profile.social_media && (
                <div>
                  <p className="text-sm text-gray-500 mb-2">Social Media</p>
                  <div className="space-y-1">
                    {profile.social_media.instagram && (
                      <p className="text-gray-900">Instagram: {profile.social_media.instagram}</p>
                    )}
                    {profile.social_media.facebook && (
                      <p className="text-gray-900">Facebook: {profile.social_media.facebook}</p>
                    )}
                    {profile.social_media.twitter && (
                      <p className="text-gray-900">Twitter: {profile.social_media.twitter}</p>
                    )}
                  </div>
                </div>
              )}
            </div>
            <button
              onClick={() => setEditing(true)}
              className="mt-6 px-6 py-2.5 text-sm font-bold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition"
            >
              Edit Profile
            </button>
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
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Public Info
                </label>
                <textarea
                  value={formData.public_info}
                  onChange={(e) => setFormData({ ...formData, public_info: e.target.value })}
                  rows={3}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Instagram
                </label>
                <input
                  type="text"
                  value={formData.instagram}
                  onChange={(e) => setFormData({ ...formData, instagram: e.target.value })}
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
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Twitter
                </label>
                <input
                  type="text"
                  value={formData.twitter}
                  onChange={(e) => setFormData({ ...formData, twitter: e.target.value })}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500"
                />
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
                    prenume: profile.prenume,
                    nume: profile.nume,
                    public_info: profile.public_info || "",
                    instagram: profile.social_media?.instagram || "",
                    facebook: profile.social_media?.facebook || "",
                    twitter: profile.social_media?.twitter || "",
                  });
                }}
                className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition"
              >
                Cancel
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}
