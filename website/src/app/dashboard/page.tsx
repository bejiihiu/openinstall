"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/Button";

interface UserInfo {
  email: string;
  role: string;
}

interface FavoriteItem {
  id: string;
  name: string;
  publisher: string;
}

interface HistoryItem {
  id: string;
  name: string;
  version: string;
  installedAt: string;
}

export default function DashboardPage() {
  const router = useRouter();
  const [user, setUser] = useState<UserInfo | null>(null);
  const [favorites, setFavorites] = useState<FavoriteItem[]>([]);
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const jwt = localStorage.getItem("jwt");
    if (!jwt) {
      router.push("/auth/login");
      return;
    }

    const fetchData = async () => {
      try {
        const headers = { Authorization: `Bearer ${jwt}` };

        const [userRes, favRes, histRes] = await Promise.all([
          fetch("/api/user/me", { headers }),
          fetch("/api/user/favorites", { headers }),
          fetch("/api/user/history", { headers }),
        ]);

        if (!userRes.ok) {
          localStorage.removeItem("jwt");
          router.push("/auth/login");
          return;
        }

        const userData = await userRes.json();
        setUser(userData);

        if (favRes.ok) {
          const favData = await favRes.json();
          setFavorites(favData);
        }

        if (histRes.ok) {
          const histData = await histRes.json();
          setHistory(histData);
        }
      } catch {
        setError("Failed to load dashboard data.");
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [router]);

  const handleLogout = () => {
    localStorage.removeItem("jwt");
    router.push("/");
  };

  const handleRemoveFavorite = async (manifestId: string) => {
    const jwt = localStorage.getItem("jwt");
    if (!jwt) return;

    try {
      await fetch("/api/user/favorites", {
        method: "DELETE",
        headers: {
          Authorization: `Bearer ${jwt}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ manifestId }),
      });

      setFavorites((prev) => prev.filter((fav) => fav.id !== manifestId));
    } catch {
      setError("Failed to remove favorite.");
    }
  };

  if (loading) {
    return (
      <main className="max-w-[1200px] mx-auto px-6 py-16">
        <p className="text-lg">Loading...</p>
      </main>
    );
  }

  if (error) {
    return (
      <main className="max-w-[1200px] mx-auto px-6 py-16">
        <div className="border border-black p-6">
          <p>{error}</p>
        </div>
      </main>
    );
  }

  return (
    <main className="max-w-[1200px] mx-auto px-6 py-16">
      <h1 className="text-[48px] font-black tracking-tight mb-8">Dashboard</h1>

      {/* User Info Section */}
      <section className="pb-8 mb-8 border-b border-black">
        <div className="flex items-center gap-4 mb-6">
          <p className="text-lg">{user?.email}</p>
          <span className="border border-black px-2 py-0.5 text-sm">
            {user?.role}
          </span>
        </div>
        <Button variant="secondary" onClick={handleLogout}>
          Logout
        </Button>
      </section>

      {/* Favorites Section */}
      <section className="pb-8 mb-8 border-b border-black">
        <h2 className="text-2xl font-bold mb-6">Favorites</h2>
        {favorites.length === 0 ? (
          <p>No favorites yet. Browse the catalog to find apps.</p>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {favorites.map((fav) => (
              <div key={fav.id} className="border border-black p-6">
                <h3 className="font-bold text-lg mb-2">{fav.name}</h3>
                <p className="mb-4">{fav.publisher}</p>
                <Button
                  variant="secondary"
                  onClick={() => handleRemoveFavorite(fav.id)}
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
        <h2 className="text-2xl font-bold mb-6">Install History</h2>
        {history.length === 0 ? (
          <p>No install history yet.</p>
        ) : (
          <div className="space-y-4">
            {history.map((item) => (
              <div key={item.id} className="border border-black p-6">
                <div className="flex justify-between items-start">
                  <div>
                    <h3 className="font-bold text-lg">{item.name}</h3>
                    <p className="text-sm">Version: {item.version}</p>
                  </div>
                  <p className="text-sm">
                    {new Date(item.installedAt).toLocaleDateString()}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </main>
  );
}