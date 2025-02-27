import React, { createContext, useState, useContext, useEffect } from "react";
import { useNavigate } from "react-router";

type AuthContextType = {
  token: string | null;
  isAuthenticated: boolean;
  login: (token: string) => void;
  logout: () => void;
};

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const navigate = useNavigate();
  const [token, setToken] = useState<string | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    const savedToken = localStorage.getItem("token");
    if (savedToken) {
      setToken(savedToken);
      setIsAuthenticated(true);
    } else {
      const currentPath = window.location.pathname;
      if (currentPath !== "/signup" && currentPath !== "/reset-password") {
        navigate("/signin");
      }
    }
  }, []);

  const login = (newToken: string) => {
    setToken(newToken);
    setIsAuthenticated(true);
    localStorage.setItem("token", newToken);
  };

  const logout = () => {
    setToken(null);
    setIsAuthenticated(false);
    localStorage.removeItem("token");
    navigate("/signin");
  };

  return (
    <AuthContext.Provider value={{ token, isAuthenticated, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = (): AuthContextType => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
};
