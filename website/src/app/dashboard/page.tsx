"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Button from "@/components/Button";

interface User {
  email: string;
  role: string;
}

interface Favorite {
  _id: string;
  name: string;
  publisher: string;
  description: string;
  version: string;
}

interface HistoryEntry {
  _id: string;
  name: string;
  version: string;
  installedAt: string;
}

export default function DashboardPage() {
  const router = useRouter();
  const [user, setUser] = useState<User | null>(null);
  const [favorites, setFavorites] = useState<Favorite[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) {
      router.push("/auth/login");
      return;
    }

    const fetchData = async () => {
      try {
        const headers = { Authorization: `Bearer ${token}` };

        const [favRes, histRes] = await Promise.all([
          fetch("/api/user/favorites", { headers }),
          fetch("/api/user/history", { headers }),
        ]);

        if (!favRes.ok || !histRes.ok) {
          localStorage.removeItem("token");
          router.push("/auth/login");
          return;
        }

        const favData = await favRes.json();
        const histData = await histRes.json();

        setUser(favData.user ?? { email: favData.email, role: favData.role });
        setFavorites(favData.favorites ?? favData);
        setHistory(histData.history ?? histData);
      } catch {
        setError("Failed to load dashboard data.");
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [router]);

  const handleLogout = () => {
    localStorage.removeItem("token");
    router.push("/");
  };

  const handleRemoveFavorite = async (manifestId: string) => {
    const token = localStorage.getItem("token");
    if (!token) return;

    try {
      await fetch("/api/user/favorites", {
        method: "DELETE",
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ manifestId }),
      });
      setFavorites((prev) => prev.filter((f) => f._id !== manifestId));
    } catch {
      setError("Failed to remove favorite.");
    }
  };

  if (loading) {
    return (
      <div className="max-w-[1200px] mx-auto px-6 pt-24 pb-32">
        <p className="text-lg font-normal">Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="max-w-[1200px] mx-auto px-6 pt-24 pb-32">
        <div className="border border-black p-4 text-lg">{error}</div>
      </div>
    );
  }

  return (
    <div className="max-w-[1200px] mx-auto px-6 pt-24 pb-32">
      <h1
        className="text-[48px] font-black tracking-[-0.05em] mb-8"
        style={{ letterSpacing: "-0.05em" }}
      >
        Dashboard
      </h1>

      {/* User Info Section */}
      {user && (
        <section className="border-b border-black pb-8 mb-8">
          <div className="flex items-center gap-4 mb-4">
            <span className="text-lg">{user.email}</span>
            <span className="border border-black px-2 py-0.5 text-sm">
              {user.role}
            </span>
          </div>
          <Button variant="secondary" onClick={handleLogout}>
            Logout
          </Button>
        </section>
      )}

      {/* Favorites Section */}
      <section className="border-b border-black pb-8 mb-8">
        <h2
          className="text-[24px] font-bold tracking-[-0.05em] mb-6"
          style={{ letterSpacing: "-0.05em" }}
        >
          Favorites
        </h2>
        {favorites.length === 0 ? (
          <p>No favorites yet. Browse the catalog to find apps.</p>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {favorites.map((app) => (
              <div
                key={app._id}
                className="border border-black p-6 flex flex-col gap-2"
              >
                <h3 className="font-bold text-lg">{app.name}</h3>
                <p className="text-sm">{app.publisher}</p>
                <Button
                  variant="secondary"
                  onClick={() => handleRemoveFavorite(app._id)}
                >
                  Remove
                </Button>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Install History Section */}
      <section>
        <h2
          className="text-[24px] font-bold tracking-[-0.05em] mb-6"
          style={{ letterSpacing: "-0.05em" }}
        >
          Install History
        </h2>
        {history.length === 0 ? (
          <p>No install history yet.</p>
        ) : (
          <div className="flex flex-col gap-3">
            {history.map((entry) => (
              <div
                key={entry._id}
                className="border border-black p-4 flex items-center justify-between"
              >
                <div>
                  <span className="font-bold">{entry.name}</span>
                  <span className="mx-2">-</span>
                  <span className="text-sm">{entry.version}</span>
                </div>
                <span className="text-sm">
                  {new Date(entry.installedAt).toLocaleDateString()}
                </span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
