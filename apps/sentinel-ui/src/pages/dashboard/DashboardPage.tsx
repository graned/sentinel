import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  AreaChart,
  Area,
  BarChart,
  Bar,
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { adminApi } from "../../api/admin";
import { authApi } from "../../api/auth";
import { Card } from "../../components/ui/Card";
import { StatCard } from "../../components/ui/StatCard";
import { Badge } from "../../components/ui/Badge";

import bgDecoration from "../../assets/bg-decoration.svg";
import styles from "./DashboardPage.module.css";

// ─── Hero shield icon (unchanged) ────────────────────────────────────────────

function HeroShieldIcon() {
  return (
    <svg
      className={styles.hShieldSvg}
      viewBox="0 0 120 140"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <defs>
        <linearGradient id="heroShieldGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#06b6d4" />
          <stop offset="100%" stopColor="#3b82f6" />
        </linearGradient>
        <linearGradient id="heroShieldInner" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="rgba(6,182,212,0.15)" />
          <stop offset="100%" stopColor="rgba(59,130,246,0.08)" />
        </linearGradient>
      </defs>
      <path
        d="M60 4L8 26v42c0 31.4 22.1 60.8 52 68 29.9-7.2 52-36.6 52-68V26L60 4z"
        fill="url(#heroShieldInner)"
        stroke="url(#heroShieldGrad)"
        strokeWidth="2"
      />
      <path
        d="M60 18L22 36v32c0 22.8 16.2 44.1 38 49.4C81.8 112.1 98 90.8 98 68V36L60 18z"
        fill="url(#heroShieldInner)"
        stroke="url(#heroShieldGrad)"
        strokeWidth="1"
        strokeOpacity="0.5"
      />
      <rect x="46" y="66" width="28" height="22" rx="4" fill="url(#heroShieldGrad)" />
      <path
        d="M50 66v-6a10 10 0 0120 0v6"
        stroke="url(#heroShieldGrad)"
        strokeWidth="3.5"
        strokeLinecap="round"
        fill="none"
      />
      <circle cx="60" cy="75" r="3.5" fill="#070d1a" />
      <rect x="58.5" y="75" width="3" height="6" rx="1.5" fill="#070d1a" />
    </svg>
  );
}

// ─── Recharts shared theme ────────────────────────────────────────────────────

const CHART_GRID_COLOR = "rgba(255,255,255,0.06)";
const CHART_TICK_STYLE = { fill: "#94a3b8", fontSize: 11 };
const CHART_TOOLTIP_STYLE = {
  background: "#111d35",
  border: "1px solid rgba(255,255,255,0.1)",
  borderRadius: "10px",
  color: "#f1f5f9",
  fontSize: 12,
};
const CHART_TOOLTIP_CURSOR = { fill: "rgba(255,255,255,0.03)" };
const PIE_COLORS = ["#06b6d4", "#1e3a5f"];

// ─── Date formatter ────────────────────────────────────────────────────────────

function fmtDate(isoDate: string): string {
  const d = new Date(isoDate + "T00:00:00");
  return d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

// ─── Local InsightCard component ──────────────────────────────────────────────

interface InsightCardProps {
  label: string;
  value: string | number;
  sub?: string;
  trend?: "up" | "down" | "neutral";
  comingSoon?: boolean;
}

function InsightCard({ label, value, sub, trend, comingSoon }: InsightCardProps) {
  return (
    <div className={styles.insightCard}>
      {comingSoon && <span className={styles.comingSoon}>Soon</span>}
      <span className={styles.insightValue}>{comingSoon ? "—" : value}</span>
      <span className={styles.insightLabel}>{label}</span>
      {sub && !comingSoon && (
        <span
          className={`${styles.insightSub} ${trend === "up"
              ? styles.trendUp
              : trend === "down"
                ? styles.trendDown
                : styles.trendNeutral
            }`}
        >
          {sub}
        </span>
      )}
      {comingSoon && (
        <span className={styles.insightSub}>Requires audit log</span>
      )}
    </div>
  );
}

// ─── Section header component ─────────────────────────────────────────────────

interface SectionHeaderProps {
  title: string;
  range: "week" | "month";
  onRangeChange: (r: "week" | "month") => void;
}

function SectionHeader({ title, range, onRangeChange }: SectionHeaderProps) {
  return (
    <div className={styles.sectionHeader}>
      <div className={styles.sectionTitleWrap}>
        <span className={styles.sectionPill}>Analytics</span>
        <h2 className={styles.sectionTitle}>{title}</h2>
      </div>
      <div className={styles.rangeToggle} role="group" aria-label="Time range">
        <button
          className={`${styles.rangeBtn} ${range === "week" ? styles.rangeBtnActive : ""}`}
          onClick={() => onRangeChange("week")}
        >
          This Week
        </button>
        <button
          className={`${styles.rangeBtn} ${range === "month" ? styles.rangeBtnActive : ""}`}
          onClick={() => onRangeChange("month")}
        >
          This Month
        </button>
      </div>
    </div>
  );
}

// ─── Chart card wrapper ────────────────────────────────────────────────────────

interface ChartCardProps {
  title: string;
  subtitle?: string;
  children: React.ReactNode;
  className?: string;
}

function ChartCard({ title, subtitle, children, className }: ChartCardProps) {
  return (
    <div className={`${styles.chartCard} ${className ?? ""}`}>
      <div className={styles.chartCardHeader}>
        <span className={styles.chartCardTitle}>{title}</span>
        {subtitle && <span className={styles.chartCardSub}>{subtitle}</span>}
      </div>
      <div className={styles.chartCardBody}>{children}</div>
    </div>
  );
}

// ─── Main page ────────────────────────────────────────────────────────────────

export function DashboardPage() {
  const [range, setRange] = useState<"week" | "month">("week");

  const { data: me } = useQuery({
    queryKey: ["me"],
    queryFn: () => authApi.getMe(),
  });

  // ── Insights queries ───────────────────────────────────────────────────────
  const { data: summary } = useQuery({
    queryKey: ["insights-summary"],
    queryFn: () => adminApi.getInsightsSummary(),
  });

  const { data: userGrowthRaw } = useQuery({
    queryKey: ["insights-user-growth"],
    queryFn: () => adminApi.getUserGrowth(30),
  });

  const { data: sessionActivityRaw } = useQuery({
    queryKey: ["insights-session-activity"],
    queryFn: () => adminApi.getSessionActivity(30),
  });

  // Transform API data → chart-friendly shapes
  const growthChartData = (userGrowthRaw ?? []).map((p) => ({
    date: fmtDate(p.date),
    total: p.total_users,
    new: p.new_users,
  }));

  const sessionChartData = (sessionActivityRaw ?? []).slice(-14).map((p) => ({
    date: fmtDate(p.date),
    sessions: p.sessions_created,
    users: p.unique_users,
  }));

  const loginActivityData = (sessionActivityRaw ?? []).map((p) => ({
    date: fmtDate(p.date),
    sessions: p.sessions_created,
  }));

  const securityBreakdown = summary
    ? [
      { name: "MFA Enabled", value: summary.mfa_adoption_pct },
      { name: "No MFA", value: 100 - summary.mfa_adoption_pct },
    ]
    : [];

  // Derived KPI values based on selected range
  const activeUsers =
    range === "week"
      ? (summary?.active_users_week ?? 0)
      : (summary?.active_users_month ?? 0);
  const newUsers =
    range === "week"
      ? (summary?.new_users_week ?? 0)
      : (summary?.new_users_month ?? 0);

  return (
    <div className={styles.page}>

      {/* ── Hero (unchanged) ──────────────────────────────────────── */}
      <div className={styles.hero}>
        <img
          src={bgDecoration}
          className={styles.bgDecoration}
          alt=""
          aria-hidden="true"
        />

        <div className={styles.heroContent}>
          <div className={styles.heroPillRow}>
            <div className={styles.heroPill}>Security Platform</div>
            <div className={styles.heroStatus}>
              <span className={styles.statusDot} />
              Operational
            </div>
          </div>
          <h1 className={styles.heroTitle}>
            Security Infrastructure<br />for Developers
          </h1>
          <p className={styles.heroSubtitle}>
            The all-in-one authentication and authorization platform.
            {me && (
              <> Signed in as <span className={styles.heroEmail}>{me.email}</span>.</>
            )}
          </p>
          <a href="/roles" className={styles.heroBtn}>
            Get Started
          </a>
        </div>

        <div className={styles.heroVisual} aria-hidden="true">
          <div className={`${styles.hRing} ${styles.hRing1}`} />
          <div className={`${styles.hRing} ${styles.hRing2}`} />
          <div className={`${styles.hRing} ${styles.hRing3}`} />
          <div className={`${styles.hOrbitTrack} ${styles.hOrbitOuter}`}>
            <span className={styles.hOrbitDot} />
          </div>
          <div className={`${styles.hOrbitTrack} ${styles.hOrbitOuter} ${styles.hOrbitPhase}`}>
            <span className={styles.hOrbitDot} />
          </div>
          <div className={`${styles.hOrbitTrack} ${styles.hOrbitMid}`}>
            <span className={styles.hOrbitDot} />
          </div>
          <div className={`${styles.hOrbitTrack} ${styles.hOrbitMid} ${styles.hOrbitPhase2}`}>
            <span className={styles.hOrbitDot} />
          </div>
          <div className={styles.hShieldWrap}>
            <HeroShieldIcon />
          </div>
        </div>
      </div>

      {/* ════════════════════════════════════════════════════════════ */}
      {/* ── Platform Insights (new section) ──────────────────────── */}
      {/* ════════════════════════════════════════════════════════════ */}

      <section className={styles.insightsSection}>

        {/* Section header + time-range toggle */}
        <SectionHeader
          title="Platform Insights"
          range={range}
          onRangeChange={setRange}
        />

        {/* ── Row 1: Primary KPIs ─────────────────────────────────── */}
        <div className={styles.insightGrid5}>
          <InsightCard
            label="Total Users"
            value={(summary?.total_users ?? 0).toLocaleString()}
            sub="All registered accounts"
            trend="neutral"
          />
          <InsightCard
            label="Active Users"
            value={activeUsers.toLocaleString()}
            sub={range === "week" ? "Last 7 days" : "Last 30 days"}
            trend="up"
          />
          <InsightCard
            label="New Users"
            value={newUsers.toLocaleString()}
            sub={range === "week" ? "Last 7 days" : "Last 30 days"}
            trend="up"
          />
          <InsightCard
            label="Active Sessions"
            value={(summary?.active_sessions ?? 0).toLocaleString()}
            sub="Live right now"
            trend="neutral"
          />
          <InsightCard
            label="MFA Adoption"
            value={summary ? `${summary.mfa_adoption_pct.toFixed(1)}%` : "—"}
            sub="Users with TOTP enabled"
            trend={summary && summary.mfa_adoption_pct >= 50 ? "up" : "neutral"}
          />
        </div>

        {/* ── Row 2: Security / Quality KPIs ─────────────────────── */}
        <div className={styles.insightGrid4}>
          <InsightCard
            label="Email Verified"
            value={summary ? `${summary.email_verified_pct.toFixed(1)}%` : "—"}
            sub="Verified user identities"
            trend="up"
          />
          <InsightCard
            label="Success Rate"
            value="—"
            comingSoon
          />
          <InsightCard
            label="Failed Logins"
            value="—"
            comingSoon
          />
          <InsightCard
            label="New Users (Month)"
            value={(summary?.new_users_month ?? 0).toLocaleString()}
            sub="Last 30 days"
            trend="up"
          />
        </div>

        {/* ── Row 3: User Growth + Session Activity ──────────────── */}
        <div className={styles.chartRow2}>
          <ChartCard
            title="User Growth"
            subtitle="Cumulative registered users — last 30 days"
            className={styles.chartWide}
          >
            <ResponsiveContainer width="100%" height={220}>
              <AreaChart data={growthChartData} margin={{ top: 4, right: 8, left: -16, bottom: 0 }}>
                <defs>
                  <linearGradient id="gradTotal" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#06b6d4" stopOpacity={0.25} />
                    <stop offset="95%" stopColor="#06b6d4" stopOpacity={0.03} />
                  </linearGradient>
                </defs>
                <CartesianGrid vertical={false} stroke={CHART_GRID_COLOR} />
                <XAxis
                  dataKey="date"
                  tick={CHART_TICK_STYLE}
                  tickLine={false}
                  axisLine={false}
                  interval={4}
                />
                <YAxis
                  tick={CHART_TICK_STYLE}
                  tickLine={false}
                  axisLine={false}
                  domain={["auto", "auto"]}
                />
                <Tooltip
                  contentStyle={CHART_TOOLTIP_STYLE}
                  cursor={{ stroke: "rgba(6,182,212,0.2)", strokeWidth: 1 }}
                />
                <Area
                  type="monotone"
                  dataKey="total"
                  name="Total Users"
                  stroke="#06b6d4"
                  strokeWidth={2}
                  fill="url(#gradTotal)"
                  dot={false}
                  activeDot={{ r: 4, fill: "#06b6d4" }}
                />
              </AreaChart>
            </ResponsiveContainer>
          </ChartCard>

          <ChartCard
            title="Session Activity"
            subtitle="Sessions created per day — last 14 days"
            className={styles.chartNarrow}
          >
            <ResponsiveContainer width="100%" height={220}>
              <BarChart data={sessionChartData} margin={{ top: 4, right: 8, left: -16, bottom: 0 }}>
                <CartesianGrid vertical={false} stroke={CHART_GRID_COLOR} />
                <XAxis
                  dataKey="date"
                  tick={CHART_TICK_STYLE}
                  tickLine={false}
                  axisLine={false}
                  interval={2}
                />
                <YAxis
                  tick={CHART_TICK_STYLE}
                  tickLine={false}
                  axisLine={false}
                />
                <Tooltip contentStyle={CHART_TOOLTIP_STYLE} cursor={CHART_TOOLTIP_CURSOR} />
                <Bar dataKey="sessions" name="Sessions" fill="#3b82f6" radius={[3, 3, 0, 0]} />
                <Bar dataKey="users" name="Unique Users" fill="#06b6d4" radius={[3, 3, 0, 0]} opacity={0.7} />
              </BarChart>
            </ResponsiveContainer>
          </ChartCard>
        </div>

        {/* ── Row 4: Daily Logins (full width) ───────────────────── */}
        <ChartCard
          title="Daily Login Activity"
          subtitle="Sessions initiated per day — last 30 days"
        >
          <ResponsiveContainer width="100%" height={200}>
            <AreaChart data={loginActivityData} margin={{ top: 4, right: 8, left: -16, bottom: 0 }}>
              <defs>
                <linearGradient id="gradSessions" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#3b82f6" stopOpacity={0.03} />
                </linearGradient>
              </defs>
              <CartesianGrid vertical={false} stroke={CHART_GRID_COLOR} />
              <XAxis
                dataKey="date"
                tick={CHART_TICK_STYLE}
                tickLine={false}
                axisLine={false}
                interval={4}
              />
              <YAxis
                tick={CHART_TICK_STYLE}
                tickLine={false}
                axisLine={false}
              />
              <Tooltip
                contentStyle={CHART_TOOLTIP_STYLE}
                cursor={{ stroke: "rgba(59,130,246,0.2)", strokeWidth: 1 }}
              />
              <Area
                type="monotone"
                dataKey="sessions"
                name="Sessions"
                stroke="#3b82f6"
                strokeWidth={2}
                fill="url(#gradSessions)"
                dot={false}
                activeDot={{ r: 4, fill: "#3b82f6" }}
              />
            </AreaChart>
          </ResponsiveContainer>
        </ChartCard>

        {/* ── Row 5: Security Breakdown + New Registrations ──────── */}
        <div className={styles.chartRow2}>
          <ChartCard title="Security Posture" subtitle="MFA adoption across all users">
            <div className={styles.pieWrap}>
              <ResponsiveContainer width="100%" height={200}>
                <PieChart>
                  <Pie
                    data={securityBreakdown}
                    cx="50%"
                    cy="50%"
                    innerRadius={55}
                    outerRadius={80}
                    paddingAngle={3}
                    dataKey="value"
                    strokeWidth={0}
                  >
                    {securityBreakdown.map((entry, index) => (
                      <Cell key={entry.name} fill={PIE_COLORS[index]} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={CHART_TOOLTIP_STYLE}
                    formatter={(v) => [`${Number(v).toFixed(1)}%`, ""]}
                  />
                  <Legend
                    iconType="circle"
                    iconSize={8}
                    formatter={(value) => (
                      <span style={{ color: "#94a3b8", fontSize: 12 }}>{value}</span>
                    )}
                  />
                </PieChart>
              </ResponsiveContainer>
              <div className={styles.pieCenterLabel}>
                <span className={styles.pieCenterValue}>
                  {summary ? `${summary.mfa_adoption_pct.toFixed(1)}%` : "—"}
                </span>
                <span className={styles.pieCenterSub}>MFA</span>
              </div>
            </div>
          </ChartCard>

          <ChartCard
            title="New Registrations"
            subtitle="Daily new user sign-ups — last 30 days"
          >
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={growthChartData} margin={{ top: 4, right: 8, left: -16, bottom: 0 }}>
                <CartesianGrid vertical={false} stroke={CHART_GRID_COLOR} />
                <XAxis
                  dataKey="date"
                  tick={CHART_TICK_STYLE}
                  tickLine={false}
                  axisLine={false}
                  interval={4}
                />
                <YAxis
                  tick={CHART_TICK_STYLE}
                  tickLine={false}
                  axisLine={false}
                />
                <Tooltip contentStyle={CHART_TOOLTIP_STYLE} cursor={CHART_TOOLTIP_CURSOR} />
                <Bar
                  dataKey="new"
                  name="New Users"
                  fill="#06b6d4"
                  radius={[3, 3, 0, 0]}
                />
              </BarChart>
            </ResponsiveContainer>
          </ChartCard>
        </div>

      </section>
    </div>
  );
}
