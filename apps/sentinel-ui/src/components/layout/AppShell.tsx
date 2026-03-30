import { useState } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { useAuth, useAuthStore } from "@sentinel/auth-react";
import {
  DashboardIcon,
  UsersIcon,
  RolesIcon,
  PoliciesIcon,
  SessionsIcon,
  TokensIcon,
  EmailIcon,
  ProvidersIcon,
  SignOutIcon,
} from "../icons";
import { UserProfilePanel } from "./UserProfilePanel";
import styles from "./AppShell.module.css";

type NavItem = {
  to: string;
  label: string;
  Icon: React.ComponentType<React.SVGProps<SVGSVGElement>>;
};

const BASE_NAV_ITEMS: NavItem[] = [
  { to: "/dashboard", label: "Dashboard", Icon: DashboardIcon },
  { to: "/roles", label: "Roles", Icon: RolesIcon },
  { to: "/policies", label: "Policies", Icon: PoliciesIcon },
  { to: "/sessions", label: "Sessions", Icon: SessionsIcon },
  { to: "/tokens", label: "API Tokens", Icon: TokensIcon },
  { to: "/email-templates", label: "Email Templates", Icon: EmailIcon },
  { to: "/providers", label: "Providers", Icon: ProvidersIcon },
];

const ADMIN_NAV_ITEM: NavItem = { to: "/users", label: "Users", Icon: UsersIcon };

export function AppShell() {
  const { logout } = useAuth();
  const { isAdmin, userEmail, firstName, lastName } = useAuthStore();
  const navigate = useNavigate();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [profileOpen, setProfileOpen] = useState(false);

  const navItems: NavItem[] = isAdmin
    ? [BASE_NAV_ITEMS[0], ADMIN_NAV_ITEM, ...BASE_NAV_ITEMS.slice(1)]
    : BASE_NAV_ITEMS;

  const handleLogout = async () => {
    await logout();
    navigate("/login");
  };

  const closeSidebar = () => setSidebarOpen(false);

  return (
    <div className={styles.shell}>
      {sidebarOpen && <div className={styles.overlay} onClick={closeSidebar} aria-hidden="true" />}

      <aside className={`${styles.sidebar} ${sidebarOpen ? styles.sidebarOpen : ""}`}>
        <div className={styles.logo}>
          <div className={styles.logoMark} aria-hidden="true">
            <svg viewBox="0 0 120 140" fill="none" xmlns="http://www.w3.org/2000/svg" width="28" height="33">
              <path
                d="M60 4L8 26v42c0 31.4 22.1 60.8 52 68 29.9-7.2 52-36.6 52-68V26L60 4z"
                fill="rgba(6,182,212,0.12)"
                stroke="#06b6d4"
                strokeWidth="2"
              />
              <path
                d="M60 18L22 36v32c0 22.8 16.2 44.1 38 49.4C81.8 112.1 98 90.8 98 68V36L60 18z"
                fill="rgba(6,182,212,0.07)"
                stroke="#06b6d4"
                strokeWidth="1"
                strokeOpacity="0.4"
              />
              <rect x="46" y="66" width="28" height="22" rx="4" fill="#06b6d4" />
              <path
                d="M50 66v-6a10 10 0 0120 0v6"
                stroke="#06b6d4"
                strokeWidth="3.5"
                strokeLinecap="round"
                fill="none"
              />
              <circle cx="60" cy="75" r="3.5" fill="#070d1a" />
              <rect x="58.5" y="75" width="3" height="6" rx="1.5" fill="#070d1a" />
            </svg>
          </div>
          <div className={styles.logoText}>
            <span className={styles.logoName}>
              Sentinel<span className={styles.logoNameAuth}>&nbsp;Auth</span>
            </span>
            <span className={styles.logoSub}>Admin</span>
          </div>
        </div>

        <nav className={styles.nav} aria-label="Main navigation">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              onClick={closeSidebar}
              className={({ isActive }) => `${styles.navLink} ${isActive ? styles.active : ""}`}
            >
              <item.Icon className={styles.navIcon} />
              {item.label}
            </NavLink>
          ))}
        </nav>

        <div className={styles.sidebarFooter}>
          {userEmail && (
            <button
              className={`${styles.userCard} ${profileOpen ? styles.userCardActive : ""}`}
              onClick={() => setProfileOpen((o) => !o)}
              aria-label="Open user profile"
            >
              <div className={styles.userAvatar}>
                {(firstName?.[0] ?? userEmail[0]).toUpperCase()}
              </div>
              <div className={styles.userInfo}>
                {(firstName || lastName) ? (
                  <span className={styles.userName}>{[firstName, lastName].filter(Boolean).join(" ")}</span>
                ) : null}
                <span className={styles.userEmail}>{userEmail}</span>
              </div>
              <svg className={styles.userCardChevron} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" aria-hidden="true">
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
          )}
          <button className={styles.logoutBtn} onClick={handleLogout}>
            <SignOutIcon className={styles.navIcon} />
            Sign out
          </button>
        </div>
      </aside>

      <div className={styles.topBar}>
        <button
          className={styles.hamburger}
          onClick={() => setSidebarOpen((o) => !o)}
          aria-label="Toggle navigation"
          aria-expanded={sidebarOpen}
        >
          ☰
        </button>
        <span className={styles.topBarTitle}>Sentinel</span>
      </div>

      <main className={styles.content}>
        <Outlet />
      </main>

      <UserProfilePanel open={profileOpen} onClose={() => setProfileOpen(false)} />
    </div>
  );
}
