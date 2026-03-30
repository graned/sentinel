import { createContext as Ae, useContext as Te, useEffect as ae, useState as p, useCallback as J, useRef as je } from "react";
import { jsxs as s, jsx as e, Fragment as H } from "react/jsx-runtime";
import { create as Ee } from "zustand";
import { persist as qe } from "zustand/middleware";
import { QueryClient as Re, QueryCache as We, useQuery as ze } from "@tanstack/react-query";
import { SentinelError as D, EmailNotVerifiedError as Fe, ForbiddenError as Oe } from "@sentinel/auth-sdk";
import { useLocation as ke, Navigate as Z, Outlet as he, useNavigate as X, Link as Q, useSearchParams as ve, Routes as Ve, Route as F } from "react-router-dom";
const we = Ae(null);
function q() {
  const t = Te(we);
  if (!t)
    throw new Error("useSentinelAuth must be called inside <SentinelAuthProvider>");
  return t;
}
const A = Ee()(
  qe(
    (t) => ({
      userId: null,
      accessToken: null,
      refreshToken: null,
      isAuthenticated: !1,
      emailVerified: !1,
      isAdmin: !1,
      mustChangePassword: !1,
      mfaSetupRequired: !1,
      userEmail: null,
      firstName: null,
      lastName: null,
      setSession: (n, r, o, i, a = !1) => t({
        userId: n,
        accessToken: r,
        refreshToken: o,
        isAuthenticated: !0,
        emailVerified: i,
        mustChangePassword: a
      }),
      setIsAdmin: (n) => t({ isAdmin: n }),
      setUserProfile: (n, r, o) => t({ userEmail: n, firstName: r, lastName: o }),
      clearMustChangePassword: () => t({ mustChangePassword: !1 }),
      setMfaSetupRequired: (n) => t({ mfaSetupRequired: n }),
      clearMfaSetupRequired: () => t({ mfaSetupRequired: !1 }),
      clearTokens: () => t({
        userId: null,
        accessToken: null,
        refreshToken: null,
        isAuthenticated: !1,
        emailVerified: !1,
        isAdmin: !1,
        mustChangePassword: !1,
        mfaSetupRequired: !1,
        userEmail: null,
        firstName: null,
        lastName: null
      })
    }),
    { name: "sentinel-auth" }
  )
);
let le = null;
function He(t) {
  le = t;
}
let ee = null;
async function De() {
  if (!le) return !1;
  if (ee) return ee;
  const t = (async () => {
    try {
      const { refreshToken: n, emailVerified: r, mustChangePassword: o } = A.getState(), i = await le.refreshSession(n);
      return A.getState().setSession(
        i.userId,
        i.accessToken,
        i.refreshToken,
        r,
        o
      ), !0;
    } catch {
      return !1;
    }
  })();
  return ee = t, t.finally(() => {
    ee = null;
  }), t;
}
function ts({
  client: t,
  redirects: n = {},
  theme: r = {},
  children: o
}) {
  ae(() => {
    He(t);
  }, [t]);
  const i = Ue(r), a = { client: t, redirects: n, theme: r };
  return /* @__PURE__ */ s(we.Provider, { value: a, children: [
    i ? /* @__PURE__ */ e("style", { children: `:root { ${i} }` }) : null,
    o
  ] });
}
function Ue(t) {
  const n = [];
  return t.primaryColor && (n.push(`--accent-primary: ${t.primaryColor};`), n.push(`--accent-primary-hover: ${t.primaryColor};`), n.push(`--border-active: ${t.primaryColor};`)), t.secondaryColor && (n.push(`--accent-blue: ${t.secondaryColor};`), n.push(`--accent-blue-hover: ${t.secondaryColor};`)), n.join(" ");
}
async function pe(t, n) {
  try {
    const o = (await t.user.getPermissions(n)).roles.some((i) => i.role_type === "admin");
    A.getState().setIsAdmin(o);
  } catch {
  }
}
function Ge() {
  const { client: t, redirects: n } = q(), { isAuthenticated: r, setSession: o, clearTokens: i, setUserProfile: a, setIsAdmin: l, setMfaSetupRequired: d } = A(), [h, u] = p(!1), [k, f] = p(null), m = J(
    async (S) => {
      u(!0), f(null);
      try {
        const N = await t.login(S);
        if (N.type === "session") {
          const y = await t.user.getMe(N.session.accessToken);
          return a(y.email, y.first_name, y.last_name), y.email_verified ? (o(
            N.session.userId,
            N.session.accessToken,
            N.session.refreshToken,
            !0,
            N.mustChangePassword
          ), N.mustChangePassword ? { success: !0, mfa: !1, mustChangePassword: !0 } : N.mfaSetupRequired ? (d(!0), { success: !0, mfa: !1, mfaSetupRequired: !0 }) : (await pe(t, N.session.accessToken), { success: !0, mfa: !1 })) : (o(
            N.session.userId,
            N.session.accessToken,
            N.session.refreshToken,
            !1
          ), { success: !0, mfa: !1, emailUnverified: !0, email: S.email });
        }
        return {
          success: !0,
          mfa: !0,
          mfaToken: N.mfaSessionToken
        };
      } catch (N) {
        const y = N?.message ?? "Login failed";
        return f(y), { success: !1, mfa: !1 };
      } finally {
        u(!1);
      }
    },
    [t, o, a, l, d]
  ), C = J(
    async (S, N) => {
      u(!0), f(null);
      try {
        const y = await t.mfa.verify({ mfa_session_token: S, code: N }), b = await t.user.getMe(y.accessToken);
        return a(b.email, b.first_name, b.last_name), o(y.userId, y.accessToken, y.refreshToken, !0), await pe(t, y.accessToken), { success: !0 };
      } catch (y) {
        const b = y?.message ?? "Verification failed";
        return f(b), { success: !1 };
      } finally {
        u(!1);
      }
    },
    [t, o, a, l]
  ), w = J(async () => {
    const { userId: S } = A.getState();
    try {
      S && await t.logout(S);
    } finally {
      i(), window.location.href = n.afterLogout ?? "/login";
    }
  }, [t, i, n.afterLogout]);
  return { isAuthenticated: r, isLoading: h, error: k, login: m, verifyMfa: C, logout: w };
}
function ns(t) {
  const n = t?.afterLogout ?? t?.login ?? "/login", r = t?.verifyEmail ?? "/verify-email", o = t?.changePassword ?? "/change-password", i = t?.unauthorized ?? "/unauthorized", a = new Re({
    defaultOptions: {
      queries: {
        retry: (l, d) => d instanceof D && d.statusCode === 401 ? !1 : l < 1,
        staleTime: 3e4
      }
    },
    queryCache: new We({
      onError: (l) => {
        l instanceof D && (l.statusCode === 401 ? (async () => await De() ? a.invalidateQueries() : (A.getState().clearTokens(), window.location.href = n))() : l instanceof Fe ? window.location.href = r : l.statusCode === 403 && l.code === "MUST_CHANGE_PASSWORD" ? window.location.href = o : l.statusCode === 403 && (window.location.href = i));
      }
    })
  });
  return a;
}
function Ye() {
  const { redirects: t } = q(), n = A((h) => h.isAuthenticated), r = A((h) => h.emailVerified), o = A((h) => h.mustChangePassword), { pathname: i } = ke(), a = t.login ?? "/login", l = t.verifyEmail ?? "/verify-email", d = t.changePassword ?? "/change-password";
  return n ? r ? o && i !== d ? /* @__PURE__ */ e(Z, { to: d, replace: !0 }) : /* @__PURE__ */ e(he, {}) : /* @__PURE__ */ e(Z, { to: l, replace: !0 }) : /* @__PURE__ */ e(Z, { to: a, replace: !0 });
}
function Ze() {
  const { redirects: t } = q(), n = A((d) => d.isAuthenticated), r = A((d) => d.emailVerified), o = A((d) => d.mustChangePassword), i = t.afterLogin ?? "/dashboard", a = t.verifyEmail ?? "/verify-email", l = t.changePassword ?? "/change-password";
  return n && r && o ? /* @__PURE__ */ e(Z, { to: l, replace: !0 }) : n && r ? /* @__PURE__ */ e(Z, { to: i, replace: !0 }) : n && !r ? /* @__PURE__ */ e(Z, { to: a, replace: !0 }) : /* @__PURE__ */ e(he, {});
}
function rs() {
  const { client: t, redirects: n } = q(), r = A((d) => d.accessToken) ?? "", o = n.unauthorized ?? "/unauthorized", { data: i, isPending: a, error: l } = ze({
    queryKey: ["authz-check", r],
    queryFn: () => t.authenticateAndAuthorize({
      access_token: r,
      method: "GET",
      path: "/v1/api/admin/roles"
    }),
    retry: !1,
    staleTime: 300 * 1e3,
    enabled: !!r
  });
  return a ? null : l instanceof Oe ? /* @__PURE__ */ e(Z, { to: o, replace: !0 }) : l || !i ? null : /* @__PURE__ */ e(he, {});
}
const Xe = "_btn_w84jm_1", Ke = "_sm_w84jm_33", Je = "_md_w84jm_39", Qe = "_primary_w84jm_44", et = "_secondary_w84jm_55", tt = "_danger_w84jm_66", nt = "_ghost_w84jm_77", rt = "_spinner_w84jm_90", ot = "_spin_w84jm_90", te = {
  btn: Xe,
  sm: Ke,
  md: Je,
  primary: Qe,
  secondary: et,
  danger: tt,
  ghost: nt,
  spinner: rt,
  spin: ot
};
function B({
  variant: t = "primary",
  size: n = "md",
  loading: r,
  children: o,
  disabled: i,
  className: a,
  ...l
}) {
  return /* @__PURE__ */ s(
    "button",
    {
      className: `${te.btn} ${te[t]} ${te[n]} ${a ?? ""}`,
      disabled: i || r,
      ...l,
      children: [
        r ? /* @__PURE__ */ e("span", { className: te.spinner }) : null,
        o
      ]
    }
  );
}
const st = "_brandPanel_1dqq3_2", it = "_gridDots_1dqq3_15", at = "_animationStage_1dqq3_34", lt = "_ring_1dqq3_44", ct = "_ring1_1dqq3_51", dt = "_ring2_1dqq3_52", ht = "_ring3_1dqq3_53", ut = "_orbitTrack_1dqq3_56", mt = "_orbitDot_1dqq3_67", ft = "_orbitOuter_1dqq3_76", pt = "_orbitPhase_1dqq3_77", gt = "_orbitMid_1dqq3_78", _t = "_orbitPhase2_1dqq3_79", kt = "_iconWrap_1dqq3_91", vt = "_iconSvg_1dqq3_102", wt = "_logoImg_1dqq3_109", yt = "_wordmark_1dqq3_119", Ct = "_wordmarkName_1dqq3_128", Nt = "_wordmarkAuth_1dqq3_129", bt = "_tagline_1dqq3_131", xt = "_taglineSubtext_1dqq3_141", x = {
  brandPanel: st,
  gridDots: it,
  animationStage: at,
  ring: lt,
  ring1: ct,
  ring2: dt,
  ring3: ht,
  orbitTrack: ut,
  orbitDot: mt,
  orbitOuter: ft,
  orbitPhase: pt,
  orbitMid: gt,
  orbitPhase2: _t,
  iconWrap: kt,
  iconSvg: vt,
  logoImg: wt,
  wordmark: yt,
  wordmarkName: Ct,
  wordmarkAuth: Nt,
  tagline: bt,
  taglineSubtext: xt
};
function Lt() {
  return /* @__PURE__ */ s(
    "svg",
    {
      className: x.iconSvg,
      viewBox: "0 0 120 140",
      fill: "none",
      xmlns: "http://www.w3.org/2000/svg",
      "aria-hidden": "true",
      children: [
        /* @__PURE__ */ s("defs", { children: [
          /* @__PURE__ */ s("linearGradient", { id: "brandShieldGrad", x1: "0%", y1: "0%", x2: "100%", y2: "100%", children: [
            /* @__PURE__ */ e("stop", { offset: "0%", stopColor: "#06b6d4" }),
            /* @__PURE__ */ e("stop", { offset: "100%", stopColor: "#3b82f6" })
          ] }),
          /* @__PURE__ */ s("linearGradient", { id: "brandShieldInner", x1: "0%", y1: "0%", x2: "100%", y2: "100%", children: [
            /* @__PURE__ */ e("stop", { offset: "0%", stopColor: "rgba(6,182,212,0.15)" }),
            /* @__PURE__ */ e("stop", { offset: "100%", stopColor: "rgba(59,130,246,0.08)" })
          ] })
        ] }),
        /* @__PURE__ */ e(
          "path",
          {
            d: "M60 4L8 26v42c0 31.4 22.1 60.8 52 68 29.9-7.2 52-36.6 52-68V26L60 4z",
            fill: "url(#brandShieldInner)",
            stroke: "url(#brandShieldGrad)",
            strokeWidth: "2"
          }
        ),
        /* @__PURE__ */ e(
          "path",
          {
            d: "M60 18L22 36v32c0 22.8 16.2 44.1 38 49.4C81.8 112.1 98 90.8 98 68V36L60 18z",
            fill: "url(#brandShieldInner)",
            stroke: "url(#brandShieldGrad)",
            strokeWidth: "1",
            strokeOpacity: "0.5"
          }
        ),
        /* @__PURE__ */ e("rect", { x: "46", y: "66", width: "28", height: "22", rx: "4", fill: "url(#brandShieldGrad)" }),
        /* @__PURE__ */ e(
          "path",
          {
            d: "M50 66v-6a10 10 0 0120 0v6",
            stroke: "url(#brandShieldGrad)",
            strokeWidth: "3.5",
            strokeLinecap: "round",
            fill: "none"
          }
        ),
        /* @__PURE__ */ e("circle", { cx: "60", cy: "75", r: "3.5", fill: "#070d1a" }),
        /* @__PURE__ */ e("rect", { x: "58.5", y: "75", width: "3", height: "6", rx: "1.5", fill: "#070d1a" })
      ]
    }
  );
}
function G({
  tagline: t,
  taglineSubtext: n,
  defaultIcon: r,
  showOrbits: o = !0
}) {
  const { theme: i } = q(), a = i.appName ?? "Sentinel", l = i.tagline ?? t;
  let d;
  return i.logo == null ? d = r ?? /* @__PURE__ */ e(Lt, {}) : typeof i.logo == "string" ? d = /* @__PURE__ */ e("img", { src: i.logo, alt: a, className: x.logoImg }) : d = i.logo, /* @__PURE__ */ s("div", { className: x.brandPanel, children: [
    /* @__PURE__ */ e("div", { className: x.gridDots, "aria-hidden": "true" }),
    /* @__PURE__ */ s("div", { className: x.animationStage, "aria-hidden": "true", children: [
      /* @__PURE__ */ e("div", { className: `${x.ring} ${x.ring1}` }),
      /* @__PURE__ */ e("div", { className: `${x.ring} ${x.ring2}` }),
      /* @__PURE__ */ e("div", { className: `${x.ring} ${x.ring3}` }),
      o && /* @__PURE__ */ s(H, { children: [
        /* @__PURE__ */ e("div", { className: `${x.orbitTrack} ${x.orbitOuter}`, children: /* @__PURE__ */ e("span", { className: x.orbitDot }) }),
        /* @__PURE__ */ e("div", { className: `${x.orbitTrack} ${x.orbitOuter} ${x.orbitPhase}`, children: /* @__PURE__ */ e("span", { className: x.orbitDot }) }),
        /* @__PURE__ */ e("div", { className: `${x.orbitTrack} ${x.orbitMid}`, children: /* @__PURE__ */ e("span", { className: x.orbitDot }) }),
        /* @__PURE__ */ e("div", { className: `${x.orbitTrack} ${x.orbitMid} ${x.orbitPhase2}`, children: /* @__PURE__ */ e("span", { className: x.orbitDot }) })
      ] }),
      /* @__PURE__ */ e("div", { className: x.iconWrap, children: d })
    ] }),
    /* @__PURE__ */ s("div", { className: x.wordmark, children: [
      /* @__PURE__ */ e("span", { className: x.wordmarkName, children: a }),
      /* @__PURE__ */ e("span", { className: x.wordmarkAuth, children: " Auth" })
    ] }),
    /* @__PURE__ */ e("p", { className: x.tagline, children: l }),
    n && /* @__PURE__ */ e("p", { className: x.taglineSubtext, children: n })
  ] });
}
const Pt = "_page_1qkuj_2", St = "_formPanel_1qkuj_173", It = "_topControls_1qkuj_184", Bt = "_topControlBtn_1qkuj_197", $t = "_topControlChevron_1qkuj_204", Mt = "_formCard_1qkuj_212", At = "_formHeader_1qkuj_220", Tt = "_formTitle_1qkuj_224", jt = "_formSubtitle_1qkuj_232", Et = "_form_1qkuj_173", qt = "_fieldWrap_1qkuj_246", Rt = "_fieldIcon_1qkuj_252", Wt = "_fieldInput_1qkuj_262", zt = "_forgotRow_1qkuj_287", Ft = "_forgotLink_1qkuj_293", Ot = "_error_1qkuj_306", Vt = "_submitBtn_1qkuj_317", Ht = "_mfaInput_1qkuj_327", Dt = "_backLink_1qkuj_335", Ut = "_signupLine_1qkuj_352", Gt = "_signupLink_1qkuj_359", Yt = "_copyright_1qkuj_372", v = {
  page: Pt,
  formPanel: St,
  topControls: It,
  topControlBtn: Bt,
  topControlChevron: $t,
  formCard: Mt,
  formHeader: At,
  formTitle: Tt,
  formSubtitle: jt,
  form: Et,
  fieldWrap: qt,
  fieldIcon: Rt,
  fieldInput: Wt,
  forgotRow: zt,
  forgotLink: Ft,
  error: Ot,
  submitBtn: Vt,
  mfaInput: Ht,
  backLink: Dt,
  signupLine: Ut,
  signupLink: Gt,
  copyright: Yt
};
function Zt() {
  const { redirects: t, theme: n } = q(), [r, o] = p(""), [i, a] = p(""), [l, d] = p(null), [h, u] = p(""), { login: k, verifyMfa: f, isLoading: m, error: C } = Ge(), w = X(), S = async (b) => {
    b.preventDefault();
    const I = await k({ email: r, password: i });
    if (I.success) {
      if (I.mfa) {
        d(I.mfaToken);
        return;
      }
      if ("emailUnverified" in I && I.emailUnverified) {
        w(t.verifyEmail ?? "/verify-email", { state: { email: I.email } });
        return;
      }
      if ("mustChangePassword" in I && I.mustChangePassword) {
        w(t.changePassword ?? "/change-password");
        return;
      }
      if ("mfaSetupRequired" in I && I.mfaSetupRequired) {
        w(t.setupMfa ?? "/setup-mfa");
        return;
      }
      w(t.afterLogin ?? "/dashboard");
    }
  }, N = async (b) => {
    b.preventDefault(), (await f(l, h)).success && w(t.afterLogin ?? "/dashboard");
  }, y = n.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";
  return /* @__PURE__ */ s("div", { className: v.page, children: [
    /* @__PURE__ */ e(G, { tagline: "Secure. Fast. Reliable." }),
    /* @__PURE__ */ s("div", { className: v.formPanel, children: [
      /* @__PURE__ */ s("div", { className: v.topControls, "aria-hidden": "true", children: [
        /* @__PURE__ */ e("span", { className: v.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
          /* @__PURE__ */ e("line", { x1: "2", y1: "12", x2: "22", y2: "12" }),
          /* @__PURE__ */ e("path", { d: "M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: v.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("line", { x1: "3", y1: "6", x2: "21", y2: "6" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "12", x2: "21", y2: "12" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "18", x2: "21", y2: "18" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: v.topControlChevron, children: /* @__PURE__ */ e("svg", { width: "12", height: "12", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2", strokeLinecap: "round", strokeLinejoin: "round", children: /* @__PURE__ */ e("polyline", { points: "6 9 12 15 18 9" }) }) })
      ] }),
      /* @__PURE__ */ e("div", { className: v.formCard, children: l ? /* @__PURE__ */ s(H, { children: [
        /* @__PURE__ */ s("div", { className: v.formHeader, children: [
          /* @__PURE__ */ e("h1", { className: v.formTitle, children: "Two-factor authentication" }),
          /* @__PURE__ */ e("p", { className: v.formSubtitle, children: "Enter the 6-digit code from your authenticator app." })
        ] }),
        /* @__PURE__ */ s("form", { onSubmit: N, className: v.form, children: [
          /* @__PURE__ */ s("div", { className: v.fieldWrap, children: [
            /* @__PURE__ */ e("span", { className: v.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
              /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
            ] }) }),
            /* @__PURE__ */ e(
              "input",
              {
                className: `${v.fieldInput} ${v.mfaInput}`,
                type: "text",
                inputMode: "numeric",
                pattern: "[0-9]{6}",
                maxLength: 6,
                value: h,
                onChange: (b) => u(b.target.value.replace(/\D/g, "")),
                placeholder: "000000",
                required: !0,
                autoFocus: !0,
                autoComplete: "one-time-code"
              }
            )
          ] }),
          C && /* @__PURE__ */ e("p", { className: v.error, children: C }),
          /* @__PURE__ */ e(B, { type: "submit", loading: m, disabled: h.length !== 6, className: v.submitBtn, children: "Verify" }),
          /* @__PURE__ */ e("button", { type: "button", className: v.backLink, onClick: () => {
            d(null), u("");
          }, children: "← Back to sign in" })
        ] })
      ] }) : /* @__PURE__ */ s(H, { children: [
        /* @__PURE__ */ s("div", { className: v.formHeader, children: [
          /* @__PURE__ */ e("h1", { className: v.formTitle, children: "Sign in to your account" }),
          /* @__PURE__ */ e("p", { className: v.formSubtitle, children: "Welcome back! Please enter your credentials." })
        ] }),
        /* @__PURE__ */ s("form", { onSubmit: S, className: v.form, children: [
          /* @__PURE__ */ s("div", { className: v.fieldWrap, children: [
            /* @__PURE__ */ e("span", { className: v.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "2", y: "4", width: "20", height: "16", rx: "2" }),
              /* @__PURE__ */ e("polyline", { points: "2,4 12,13 22,4" })
            ] }) }),
            /* @__PURE__ */ e(
              "input",
              {
                className: v.fieldInput,
                type: "email",
                value: r,
                onChange: (b) => o(b.target.value),
                required: !0,
                autoComplete: "email",
                placeholder: "user@example.com"
              }
            )
          ] }),
          /* @__PURE__ */ s("div", { className: v.fieldWrap, children: [
            /* @__PURE__ */ e("span", { className: v.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
              /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
            ] }) }),
            /* @__PURE__ */ e(
              "input",
              {
                className: v.fieldInput,
                type: "password",
                value: i,
                onChange: (b) => a(b.target.value),
                required: !0,
                autoComplete: "current-password",
                placeholder: "••••••••"
              }
            )
          ] }),
          /* @__PURE__ */ e("div", { className: v.forgotRow, children: /* @__PURE__ */ e(Q, { to: t.forgotPassword ?? "/forgot-password", className: v.forgotLink, children: "Forgot your password?" }) }),
          C && /* @__PURE__ */ e("p", { className: v.error, children: C }),
          /* @__PURE__ */ e(B, { type: "submit", loading: m, className: v.submitBtn, children: "Sign In" })
        ] }),
        /* @__PURE__ */ s("p", { className: v.signupLine, children: [
          "Don't have an account?",
          " ",
          /* @__PURE__ */ e(Q, { to: t.register ?? "/register", className: v.signupLink, children: "Sign up" })
        ] })
      ] }) }),
      /* @__PURE__ */ e("p", { className: v.copyright, children: y })
    ] })
  ] });
}
const Xt = "_page_jtikv_2", Kt = "_formPanel_jtikv_173", Jt = "_topControls_jtikv_184", Qt = "_topControlBtn_jtikv_197", en = "_topControlChevron_jtikv_204", tn = "_formCard_jtikv_212", nn = "_formHeader_jtikv_220", rn = "_formTitle_jtikv_224", on = "_formSubtitle_jtikv_232", sn = "_form_jtikv_173", an = "_nameRow_jtikv_246", ln = "_fieldWrap_jtikv_251", cn = "_fieldIcon_jtikv_263", dn = "_fieldInput_jtikv_273", hn = "_pwdChecklist_jtikv_298", un = "_pwdRule_jtikv_310", mn = "_pwdRuleMet_jtikv_319", fn = "_pwdRuleIcon_jtikv_323", pn = "_error_jtikv_330", gn = "_submitBtn_jtikv_341", _n = "_signinLine_jtikv_351", kn = "_signinLink_jtikv_358", vn = "_termsNote_jtikv_371", wn = "_termsLink_jtikv_379", yn = "_copyright_jtikv_391", _ = {
  page: Xt,
  formPanel: Kt,
  topControls: Jt,
  topControlBtn: Qt,
  topControlChevron: en,
  formCard: tn,
  formHeader: nn,
  formTitle: rn,
  formSubtitle: on,
  form: sn,
  nameRow: an,
  fieldWrap: ln,
  fieldIcon: cn,
  fieldInput: dn,
  pwdChecklist: hn,
  pwdRule: un,
  pwdRuleMet: mn,
  pwdRuleIcon: fn,
  error: pn,
  submitBtn: gn,
  signinLine: _n,
  signinLink: kn,
  termsNote: vn,
  termsLink: wn,
  copyright: yn
}, Cn = [
  { key: "length", label: "At least 12 characters", test: (t) => t.length >= 12 },
  { key: "upper", label: "At least one uppercase letter", test: (t) => /[A-Z]/.test(t) },
  { key: "lower", label: "At least one lowercase letter", test: (t) => /[a-z]/.test(t) },
  { key: "digit", label: "At least one number", test: (t) => /[0-9]/.test(t) },
  { key: "special", label: "At least one special character", test: (t) => /[^A-Za-z0-9]/.test(t) }
];
function Nn() {
  const { client: t, redirects: n, theme: r } = q(), [o, i] = p(""), [a, l] = p(""), [d, h] = p(""), [u, k] = p(""), [f, m] = p(""), [C, w] = p(!1), [S, N] = p(!1), [y, b] = p(null), I = X(), j = Cn.map((L) => ({
    ...L,
    met: L.test(u)
  })), z = j.every((L) => L.met), R = async (L) => {
    if (L.preventDefault(), b(null), !z) {
      w(!0);
      return;
    }
    if (u !== f) {
      b("Passwords do not match.");
      return;
    }
    N(!0);
    try {
      await t.register({ first_name: o, last_name: a, email: d, password: u }), I(n.verifyEmail ?? "/verify-email", { state: { email: d, fromRegistration: !0 } });
    } catch (U) {
      b(U instanceof D ? U.message : "Registration failed. Please try again.");
    } finally {
      N(!1);
    }
  }, $ = r.appName ?? "Sentinel", W = r.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";
  return /* @__PURE__ */ s("div", { className: _.page, children: [
    /* @__PURE__ */ e(G, { tagline: "Secure. Fast. Reliable." }),
    /* @__PURE__ */ s("div", { className: _.formPanel, children: [
      /* @__PURE__ */ s("div", { className: _.topControls, "aria-hidden": "true", children: [
        /* @__PURE__ */ e("span", { className: _.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
          /* @__PURE__ */ e("line", { x1: "2", y1: "12", x2: "22", y2: "12" }),
          /* @__PURE__ */ e("path", { d: "M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: _.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("line", { x1: "3", y1: "6", x2: "21", y2: "6" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "12", x2: "21", y2: "12" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "18", x2: "21", y2: "18" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: _.topControlChevron, children: /* @__PURE__ */ e("svg", { width: "12", height: "12", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2", strokeLinecap: "round", strokeLinejoin: "round", children: /* @__PURE__ */ e("polyline", { points: "6 9 12 15 18 9" }) }) })
      ] }),
      /* @__PURE__ */ s("div", { className: _.formCard, children: [
        /* @__PURE__ */ s("div", { className: _.formHeader, children: [
          /* @__PURE__ */ e("h1", { className: _.formTitle, children: "Create your account" }),
          /* @__PURE__ */ e("p", { className: _.formSubtitle, children: "Start your free trial. No credit card required." })
        ] }),
        /* @__PURE__ */ s("form", { onSubmit: R, className: _.form, children: [
          /* @__PURE__ */ s("div", { className: _.nameRow, children: [
            /* @__PURE__ */ s("div", { className: _.fieldWrap, children: [
              /* @__PURE__ */ e("span", { className: _.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
                /* @__PURE__ */ e("path", { d: "M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" }),
                /* @__PURE__ */ e("circle", { cx: "12", cy: "7", r: "4" })
              ] }) }),
              /* @__PURE__ */ e(
                "input",
                {
                  className: _.fieldInput,
                  type: "text",
                  value: o,
                  onChange: (L) => i(L.target.value),
                  required: !0,
                  autoComplete: "given-name",
                  placeholder: "First Name"
                }
              )
            ] }),
            /* @__PURE__ */ s("div", { className: _.fieldWrap, children: [
              /* @__PURE__ */ e("span", { className: _.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
                /* @__PURE__ */ e("path", { d: "M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" }),
                /* @__PURE__ */ e("circle", { cx: "12", cy: "7", r: "4" })
              ] }) }),
              /* @__PURE__ */ e(
                "input",
                {
                  className: _.fieldInput,
                  type: "text",
                  value: a,
                  onChange: (L) => l(L.target.value),
                  required: !0,
                  autoComplete: "family-name",
                  placeholder: "Last Name"
                }
              )
            ] })
          ] }),
          /* @__PURE__ */ s("div", { className: _.fieldWrap, children: [
            /* @__PURE__ */ e("span", { className: _.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "2", y: "4", width: "20", height: "16", rx: "2" }),
              /* @__PURE__ */ e("polyline", { points: "2,4 12,13 22,4" })
            ] }) }),
            /* @__PURE__ */ e(
              "input",
              {
                className: _.fieldInput,
                type: "email",
                value: d,
                onChange: (L) => h(L.target.value),
                required: !0,
                autoComplete: "email",
                placeholder: "Email"
              }
            )
          ] }),
          /* @__PURE__ */ s("div", { className: _.fieldWrap, children: [
            /* @__PURE__ */ e("span", { className: _.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
              /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
            ] }) }),
            /* @__PURE__ */ e(
              "input",
              {
                className: _.fieldInput,
                type: "password",
                value: u,
                onChange: (L) => {
                  k(L.target.value), w(!0);
                },
                autoComplete: "new-password",
                placeholder: "Password"
              }
            )
          ] }),
          C && /* @__PURE__ */ e("ul", { className: _.pwdChecklist, "aria-label": "Password requirements", children: j.map((L) => /* @__PURE__ */ s("li", { className: `${_.pwdRule} ${L.met ? _.pwdRuleMet : ""}`, children: [
            /* @__PURE__ */ e("span", { className: _.pwdRuleIcon, "aria-hidden": "true", children: L.met ? /* @__PURE__ */ e("svg", { width: "13", height: "13", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2.5", strokeLinecap: "round", strokeLinejoin: "round", children: /* @__PURE__ */ e("polyline", { points: "20 6 9 17 4 12" }) }) : /* @__PURE__ */ s("svg", { width: "13", height: "13", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2.5", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("line", { x1: "18", y1: "6", x2: "6", y2: "18" }),
              /* @__PURE__ */ e("line", { x1: "6", y1: "6", x2: "18", y2: "18" })
            ] }) }),
            L.label
          ] }, L.key)) }),
          /* @__PURE__ */ s("div", { className: _.fieldWrap, children: [
            /* @__PURE__ */ e("span", { className: _.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
              /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
            ] }) }),
            /* @__PURE__ */ e(
              "input",
              {
                className: _.fieldInput,
                type: "password",
                value: f,
                onChange: (L) => m(L.target.value),
                autoComplete: "new-password",
                placeholder: "Confirm Password"
              }
            )
          ] }),
          y && /* @__PURE__ */ e("p", { className: _.error, children: y }),
          /* @__PURE__ */ e(B, { type: "submit", loading: S, className: _.submitBtn, children: "Create Account" })
        ] }),
        /* @__PURE__ */ s("p", { className: _.signinLine, children: [
          "Already have an account?",
          " ",
          /* @__PURE__ */ e(Q, { to: n.login ?? "/login", className: _.signinLink, children: "Sign in" })
        ] }),
        /* @__PURE__ */ s("p", { className: _.termsNote, children: [
          "Your data is secured using ",
          $,
          " Auth. By signing up, you agree to the",
          " ",
          /* @__PURE__ */ e("a", { href: "#", className: _.termsLink, children: "Terms of Service" }),
          "."
        ] })
      ] }),
      /* @__PURE__ */ e("p", { className: _.copyright, children: W })
    ] })
  ] });
}
const bn = "_page_1eeri_2", xn = "_formPanel_1eeri_133", Ln = "_topControls_1eeri_144", Pn = "_topControlBtn_1eeri_157", Sn = "_topControlChevron_1eeri_164", In = "_formCard_1eeri_172", Bn = "_successBanner_1eeri_181", $n = "_formHeader_1eeri_195", Mn = "_envelopeIcon_1eeri_202", An = "_formTitle_1eeri_207", Tn = "_formSubtitle_1eeri_215", jn = "_emailHighlight_1eeri_222", En = "_fieldWrap_1eeri_228", qn = "_fieldIcon_1eeri_234", Rn = "_fieldInput_1eeri_244", Wn = "_successMsg_1eeri_267", zn = "_errorText_1eeri_277", Fn = "_submitBtn_1eeri_288", On = "_altAction_1eeri_302", Vn = "_altLink_1eeri_308", Hn = "_spamHint_1eeri_321", Dn = "_backLine_1eeri_329", Un = "_backLink_1eeri_336", Gn = "_statusCenter_1eeri_355", Yn = "_spinner_1eeri_372", Zn = "_successIcon_1eeri_382", Xn = "_errorIcon_1eeri_388", Kn = "_actionBtn_1eeri_394", Jn = "_copyright_1eeri_402", g = {
  page: bn,
  formPanel: xn,
  topControls: Ln,
  topControlBtn: Pn,
  topControlChevron: Sn,
  formCard: In,
  successBanner: Bn,
  formHeader: $n,
  envelopeIcon: Mn,
  formTitle: An,
  formSubtitle: Tn,
  emailHighlight: jn,
  fieldWrap: En,
  fieldIcon: qn,
  fieldInput: Rn,
  successMsg: Wn,
  errorText: zn,
  submitBtn: Fn,
  altAction: On,
  altLink: Vn,
  spamHint: Hn,
  backLine: Dn,
  backLink: Un,
  statusCenter: Gn,
  spinner: Yn,
  successIcon: Zn,
  errorIcon: Xn,
  actionBtn: Kn,
  copyright: Jn
}, ge = 30;
function Qn(t) {
  const [n, r] = t.split("@");
  return r ? n.length <= 2 ? `${n[0]}*@${r}` : `${n[0] + "*".repeat(n.length - 2) + n[n.length - 1]}@${r}` : t;
}
function er() {
  const { client: t, redirects: n, theme: r } = q(), o = X(), i = ke(), [a] = ve(), l = A((T) => T.clearTokens), d = i.state ?? {}, h = a.get("token"), [u, k] = p(h ? "verifying" : "pending"), f = je(!1), [m, C] = p(d.email ?? ""), [w, S] = p(""), [N, y] = p(""), [b, I] = p(""), [j, z] = p(ge), [R, $] = p(!1), W = n.login ?? "/login", L = n.register ?? "/register", U = r.copyright ?? "© 2026 Sentinel Auth. All rights reserved.";
  ae(() => {
    if (j <= 0) return;
    const T = setInterval(() => z((K) => Math.max(0, K - 1)), 1e3);
    return () => clearInterval(T);
  }, [j]), ae(() => {
    !h || f.current || (f.current = !0, t.verifyEmail(h).then(() => {
      l(), k("success");
    }).catch((T) => {
      const K = T instanceof D ? T.message : "Verification failed. The link may have expired or already been used.";
      y(K), k("error");
    }));
  }, [h]);
  const se = J(async () => {
    const T = m || w.trim();
    if (T) {
      $(!0), I("");
      try {
        await t.resendVerification({ email: T }), m || C(w.trim()), I("Verification email sent! Check your inbox."), z(ge);
      } catch (K) {
        const Me = K instanceof D ? K.message : "Failed to resend. Please try again.";
        I(Me);
      } finally {
        $(!1);
      }
    }
  }, [t, m, w]), M = () => {
    k("pending"), y("");
  };
  return /* @__PURE__ */ s("div", { className: g.page, children: [
    /* @__PURE__ */ e(G, { tagline: "Secure account verification", taglineSubtext: "We've sent a confirmation link to your email." }),
    /* @__PURE__ */ s("div", { className: g.formPanel, children: [
      /* @__PURE__ */ s("div", { className: g.topControls, "aria-hidden": "true", children: [
        /* @__PURE__ */ e("span", { className: g.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
          /* @__PURE__ */ e("line", { x1: "2", y1: "12", x2: "22", y2: "12" }),
          /* @__PURE__ */ e("path", { d: "M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: g.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("line", { x1: "3", y1: "6", x2: "21", y2: "6" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "12", x2: "21", y2: "12" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "18", x2: "21", y2: "18" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: g.topControlChevron, children: /* @__PURE__ */ e("svg", { width: "12", height: "12", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2", strokeLinecap: "round", strokeLinejoin: "round", children: /* @__PURE__ */ e("polyline", { points: "6 9 12 15 18 9" }) }) })
      ] }),
      /* @__PURE__ */ s("div", { className: g.formCard, children: [
        u === "verifying" && /* @__PURE__ */ s("div", { className: g.statusCenter, children: [
          /* @__PURE__ */ e("div", { className: g.spinner, "aria-label": "Verifying" }),
          /* @__PURE__ */ e("h1", { className: g.formTitle, children: "Verifying your email…" }),
          /* @__PURE__ */ e("p", { className: g.formSubtitle, children: "Please wait a moment." })
        ] }),
        u === "success" && /* @__PURE__ */ s("div", { className: g.statusCenter, children: [
          /* @__PURE__ */ e("div", { className: g.successIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "48", height: "48", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
            /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
            /* @__PURE__ */ e("polyline", { points: "9 12 11 14 15 10" })
          ] }) }),
          /* @__PURE__ */ e("h1", { className: g.formTitle, children: "Email verified!" }),
          /* @__PURE__ */ e("p", { className: g.formSubtitle, children: "Your email has been verified. Please sign in to continue." }),
          /* @__PURE__ */ e(B, { className: g.actionBtn, onClick: () => o(W), children: "Sign in" })
        ] }),
        u === "error" && /* @__PURE__ */ s("div", { className: g.statusCenter, children: [
          /* @__PURE__ */ e("div", { className: g.errorIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "48", height: "48", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
            /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
            /* @__PURE__ */ e("line", { x1: "15", y1: "9", x2: "9", y2: "15" }),
            /* @__PURE__ */ e("line", { x1: "9", y1: "9", x2: "15", y2: "15" })
          ] }) }),
          /* @__PURE__ */ e("h1", { className: g.formTitle, children: "Verification failed" }),
          /* @__PURE__ */ e("p", { className: g.errorText, children: N }),
          /* @__PURE__ */ e(B, { className: g.actionBtn, onClick: M, children: "Request a new link" }),
          /* @__PURE__ */ e("p", { className: g.backLine, children: /* @__PURE__ */ e("button", { type: "button", className: g.backLink, onClick: () => {
            l(), o(W);
          }, children: "Back to sign in" }) })
        ] }),
        u === "pending" && /* @__PURE__ */ s(H, { children: [
          d.fromRegistration && /* @__PURE__ */ s("div", { className: g.successBanner, role: "status", children: [
            /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2", strokeLinecap: "round", strokeLinejoin: "round", "aria-hidden": "true", children: [
              /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
              /* @__PURE__ */ e("polyline", { points: "9 12 11 14 15 10" })
            ] }),
            "Account created successfully"
          ] }),
          /* @__PURE__ */ s("div", { className: g.formHeader, children: [
            /* @__PURE__ */ e("div", { className: g.envelopeIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "40", height: "40", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.5", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "2", y: "4", width: "20", height: "16", rx: "2" }),
              /* @__PURE__ */ e("polyline", { points: "2,4 12,13 22,4" })
            ] }) }),
            /* @__PURE__ */ e("h1", { className: g.formTitle, children: "Verify your email" }),
            m ? /* @__PURE__ */ s("p", { className: g.formSubtitle, children: [
              "We sent a verification link to",
              " ",
              /* @__PURE__ */ e("strong", { className: g.emailHighlight, children: Qn(m) }),
              ".",
              " ",
              "Click it to activate your account."
            ] }) : /* @__PURE__ */ e("p", { className: g.formSubtitle, children: "Enter your email address to receive a new verification link." })
          ] }),
          !m && /* @__PURE__ */ s("div", { className: g.fieldWrap, children: [
            /* @__PURE__ */ e("span", { className: g.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "2", y: "4", width: "20", height: "16", rx: "2" }),
              /* @__PURE__ */ e("polyline", { points: "2,4 12,13 22,4" })
            ] }) }),
            /* @__PURE__ */ e("input", { className: g.fieldInput, type: "email", value: w, onChange: (T) => S(T.target.value), placeholder: "user@example.com", autoComplete: "email" })
          ] }),
          b && /* @__PURE__ */ e("p", { className: b.includes("sent") ? g.successMsg : g.errorText, children: b }),
          /* @__PURE__ */ e(B, { className: g.submitBtn, onClick: se, loading: R, disabled: j > 0 || !m && !w.trim(), children: j > 0 ? `Resend in ${j}s` : "Resend verification email" }),
          /* @__PURE__ */ e("p", { className: g.altAction, children: /* @__PURE__ */ e(Q, { to: L, className: g.altLink, children: "Use a different email" }) }),
          /* @__PURE__ */ e("p", { className: g.spamHint, children: "Didn't receive it? Check your spam folder or request a new link." }),
          /* @__PURE__ */ e("p", { className: g.backLine, children: /* @__PURE__ */ e("button", { type: "button", className: g.backLink, onClick: () => {
            l(), o(W);
          }, children: "Back to sign in" }) })
        ] })
      ] }),
      /* @__PURE__ */ e("p", { className: g.copyright, children: U })
    ] })
  ] });
}
const tr = "_page_phhud_2", nr = "_formPanel_phhud_133", rr = "_topControls_phhud_144", or = "_topControlBtn_phhud_157", sr = "_topControlChevron_phhud_164", ir = "_formCard_phhud_172", ar = "_formHeader_phhud_181", lr = "_lockIcon_phhud_188", cr = "_formTitle_phhud_193", dr = "_formSubtitle_phhud_201", hr = "_form_phhud_133", ur = "_fieldWrap_phhud_216", mr = "_fieldIcon_phhud_222", fr = "_fieldInput_phhud_232", pr = "_submitBtn_phhud_255", gr = "_backLine_phhud_264", _r = "_backLink_phhud_271", kr = "_statusCenter_phhud_289", vr = "_envelopeIcon_phhud_306", wr = "_actionBtnLink_phhud_313", yr = "_actionBtn_phhud_313", Cr = "_altAction_phhud_325", Nr = "_altLink_phhud_331", br = "_copyright_phhud_349", P = {
  page: tr,
  formPanel: nr,
  topControls: rr,
  topControlBtn: or,
  topControlChevron: sr,
  formCard: ir,
  formHeader: ar,
  lockIcon: lr,
  formTitle: cr,
  formSubtitle: dr,
  form: hr,
  fieldWrap: ur,
  fieldIcon: mr,
  fieldInput: fr,
  submitBtn: pr,
  backLine: gr,
  backLink: _r,
  statusCenter: kr,
  envelopeIcon: vr,
  actionBtnLink: wr,
  actionBtn: yr,
  altAction: Cr,
  altLink: Nr,
  copyright: br
};
function xr() {
  const { client: t, redirects: n, theme: r } = q(), o = X(), [i, a] = p("form"), [l, d] = p(""), [h, u] = p(!1), k = n.login ?? "/login", f = r.copyright ?? "© 2026 Sentinel Auth. All rights reserved.", m = async (C) => {
    C.preventDefault(), u(!0);
    try {
      await t.forgotPassword({ email: l });
    } catch {
    } finally {
      u(!1), a("sent");
    }
  };
  return /* @__PURE__ */ s("div", { className: P.page, children: [
    /* @__PURE__ */ e(G, { tagline: "Password recovery", taglineSubtext: "We'll send you a secure reset link." }),
    /* @__PURE__ */ s("div", { className: P.formPanel, children: [
      /* @__PURE__ */ s("div", { className: P.topControls, "aria-hidden": "true", children: [
        /* @__PURE__ */ e("span", { className: P.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
          /* @__PURE__ */ e("line", { x1: "2", y1: "12", x2: "22", y2: "12" }),
          /* @__PURE__ */ e("path", { d: "M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: P.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("line", { x1: "3", y1: "6", x2: "21", y2: "6" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "12", x2: "21", y2: "12" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "18", x2: "21", y2: "18" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: P.topControlChevron, children: /* @__PURE__ */ e("svg", { width: "12", height: "12", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2", strokeLinecap: "round", strokeLinejoin: "round", children: /* @__PURE__ */ e("polyline", { points: "6 9 12 15 18 9" }) }) })
      ] }),
      /* @__PURE__ */ s("div", { className: P.formCard, children: [
        i === "form" && /* @__PURE__ */ s(H, { children: [
          /* @__PURE__ */ s("div", { className: P.formHeader, children: [
            /* @__PURE__ */ e("div", { className: P.lockIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "40", height: "40", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.5", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
              /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
            ] }) }),
            /* @__PURE__ */ e("h1", { className: P.formTitle, children: "Forgot your password?" }),
            /* @__PURE__ */ e("p", { className: P.formSubtitle, children: "Enter your email address and we'll send you a link to reset your password." })
          ] }),
          /* @__PURE__ */ s("form", { onSubmit: m, className: P.form, children: [
            /* @__PURE__ */ s("div", { className: P.fieldWrap, children: [
              /* @__PURE__ */ e("span", { className: P.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
                /* @__PURE__ */ e("rect", { x: "2", y: "4", width: "20", height: "16", rx: "2" }),
                /* @__PURE__ */ e("polyline", { points: "2,4 12,13 22,4" })
              ] }) }),
              /* @__PURE__ */ e(
                "input",
                {
                  className: P.fieldInput,
                  type: "email",
                  value: l,
                  onChange: (C) => d(C.target.value),
                  required: !0,
                  autoComplete: "email",
                  placeholder: "user@example.com",
                  autoFocus: !0
                }
              )
            ] }),
            /* @__PURE__ */ e(B, { type: "submit", loading: h, className: P.submitBtn, children: "Send reset link" })
          ] }),
          /* @__PURE__ */ e("p", { className: P.backLine, children: /* @__PURE__ */ e("button", { type: "button", className: P.backLink, onClick: () => o(k), children: "Back to sign in" }) })
        ] }),
        i === "sent" && /* @__PURE__ */ s("div", { className: P.statusCenter, children: [
          /* @__PURE__ */ e("div", { className: P.envelopeIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "48", height: "48", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.5", strokeLinecap: "round", strokeLinejoin: "round", children: [
            /* @__PURE__ */ e("rect", { x: "2", y: "4", width: "20", height: "16", rx: "2" }),
            /* @__PURE__ */ e("polyline", { points: "2,4 12,13 22,4" })
          ] }) }),
          /* @__PURE__ */ e("h1", { className: P.formTitle, children: "Check your email" }),
          /* @__PURE__ */ e("p", { className: P.formSubtitle, children: "If an account exists for that email address, we've sent a password reset link. Check your inbox (and spam folder)." }),
          /* @__PURE__ */ e(Q, { to: k, className: P.actionBtnLink, children: /* @__PURE__ */ e(B, { className: P.actionBtn, children: "Back to sign in" }) }),
          /* @__PURE__ */ e("p", { className: P.altAction, children: /* @__PURE__ */ e("button", { type: "button", className: P.altLink, onClick: () => {
            d(""), a("form");
          }, children: "Try a different email" }) })
        ] })
      ] }),
      /* @__PURE__ */ e("p", { className: P.copyright, children: f })
    ] })
  ] });
}
const Lr = "_page_75zf5_2", Pr = "_formPanel_75zf5_133", Sr = "_topControls_75zf5_144", Ir = "_topControlBtn_75zf5_157", Br = "_topControlChevron_75zf5_164", $r = "_formCard_75zf5_172", Mr = "_formHeader_75zf5_181", Ar = "_lockIcon_75zf5_188", Tr = "_formTitle_75zf5_193", jr = "_formSubtitle_75zf5_201", Er = "_form_75zf5_133", qr = "_fieldWrap_75zf5_216", Rr = "_fieldIcon_75zf5_222", Wr = "_fieldInput_75zf5_232", zr = "_checklist_75zf5_255", Fr = "_checkItem_75zf5_264", Or = "_checkIcon_75zf5_270", Vr = "_met_75zf5_276", Hr = "_checkLabel_75zf5_280", Dr = "_errorText_75zf5_291", Ur = "_submitBtn_75zf5_302", Gr = "_statusCenter_75zf5_311", Yr = "_successIcon_75zf5_328", Zr = "_errorIcon_75zf5_334", Xr = "_errorTextBlock_75zf5_340", Kr = "_actionBtn_75zf5_352", Jr = "_backLine_75zf5_360", Qr = "_backLink_75zf5_367", eo = "_copyright_75zf5_385", c = {
  page: Lr,
  formPanel: Pr,
  topControls: Sr,
  topControlBtn: Ir,
  topControlChevron: Br,
  formCard: $r,
  formHeader: Mr,
  lockIcon: Ar,
  formTitle: Tr,
  formSubtitle: jr,
  form: Er,
  fieldWrap: qr,
  fieldIcon: Rr,
  fieldInput: Wr,
  checklist: zr,
  checkItem: Fr,
  checkIcon: Or,
  met: Vr,
  checkLabel: Hr,
  errorText: Dr,
  submitBtn: Ur,
  statusCenter: Gr,
  successIcon: Yr,
  errorIcon: Zr,
  errorTextBlock: Xr,
  actionBtn: Kr,
  backLine: Jr,
  backLink: Qr,
  copyright: eo
}, ye = [
  { label: "At least 12 characters", test: (t) => t.length >= 12 },
  { label: "One uppercase letter", test: (t) => /[A-Z]/.test(t) },
  { label: "One lowercase letter", test: (t) => /[a-z]/.test(t) },
  { label: "One number", test: (t) => /[0-9]/.test(t) },
  { label: "One special character", test: (t) => /[^A-Za-z0-9]/.test(t) }
];
function to({ password: t }) {
  return /* @__PURE__ */ e("ul", { className: c.checklist, "aria-label": "Password requirements", children: ye.map(({ label: n, test: r }) => {
    const o = r(t);
    return /* @__PURE__ */ s("li", { className: c.checkItem, children: [
      /* @__PURE__ */ e(
        "svg",
        {
          className: `${c.checkIcon}${o ? ` ${c.met}` : ""}`,
          width: "14",
          height: "14",
          viewBox: "0 0 24 24",
          fill: "none",
          stroke: "currentColor",
          strokeWidth: "2.5",
          strokeLinecap: "round",
          strokeLinejoin: "round",
          "aria-hidden": "true",
          children: o ? /* @__PURE__ */ e("polyline", { points: "20 6 9 17 4 12" }) : /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "9" })
        }
      ),
      /* @__PURE__ */ e("span", { className: `${c.checkLabel}${o ? ` ${c.met}` : ""}`, children: n })
    ] }, n);
  }) });
}
function no(t) {
  for (const { label: n, test: r } of ye)
    if (!r(t)) return `Password must include: ${n.toLowerCase()}.`;
  return null;
}
function ro() {
  const { client: t, redirects: n, theme: r } = q(), o = X(), [i] = ve(), a = i.get("token"), [l, d] = p(a ? "form" : "no-token"), [h, u] = p(""), [k, f] = p(""), [m, C] = p(""), [w, S] = p(""), [N, y] = p(!1), b = n.login ?? "/login", I = n.forgotPassword ?? "/forgot-password", j = r.copyright ?? "© 2026 Sentinel Auth. All rights reserved.", z = async (R) => {
    R.preventDefault(), C(""), S("");
    const $ = no(h);
    if ($) {
      C($);
      return;
    }
    if (h !== k) {
      C("Passwords do not match.");
      return;
    }
    y(!0);
    try {
      await t.resetPassword({ token: a, new_password: h }), d("success");
    } catch (W) {
      const L = W instanceof D ? W.message : "Failed to reset password. The link may have expired or already been used.";
      S(L), d("error");
    } finally {
      y(!1);
    }
  };
  return /* @__PURE__ */ s("div", { className: c.page, children: [
    /* @__PURE__ */ e(G, { tagline: "Set a new password", taglineSubtext: "Choose a strong password to secure your account." }),
    /* @__PURE__ */ s("div", { className: c.formPanel, children: [
      /* @__PURE__ */ s("div", { className: c.topControls, "aria-hidden": "true", children: [
        /* @__PURE__ */ e("span", { className: c.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
          /* @__PURE__ */ e("line", { x1: "2", y1: "12", x2: "22", y2: "12" }),
          /* @__PURE__ */ e("path", { d: "M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: c.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("line", { x1: "3", y1: "6", x2: "21", y2: "6" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "12", x2: "21", y2: "12" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "18", x2: "21", y2: "18" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: c.topControlChevron, children: /* @__PURE__ */ e("svg", { width: "12", height: "12", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2", strokeLinecap: "round", strokeLinejoin: "round", children: /* @__PURE__ */ e("polyline", { points: "6 9 12 15 18 9" }) }) })
      ] }),
      /* @__PURE__ */ s("div", { className: c.formCard, children: [
        l === "form" && /* @__PURE__ */ s(H, { children: [
          /* @__PURE__ */ s("div", { className: c.formHeader, children: [
            /* @__PURE__ */ e("div", { className: c.lockIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "40", height: "40", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.5", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
              /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
            ] }) }),
            /* @__PURE__ */ e("h1", { className: c.formTitle, children: "Reset your password" }),
            /* @__PURE__ */ e("p", { className: c.formSubtitle, children: "Enter a new password for your account. All active sessions will be signed out." })
          ] }),
          /* @__PURE__ */ s("form", { onSubmit: z, className: c.form, children: [
            /* @__PURE__ */ s("div", { children: [
              /* @__PURE__ */ s("div", { className: c.fieldWrap, children: [
                /* @__PURE__ */ e("span", { className: c.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
                  /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
                  /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
                ] }) }),
                /* @__PURE__ */ e("input", { className: c.fieldInput, type: "password", value: h, onChange: (R) => u(R.target.value), required: !0, autoComplete: "new-password", placeholder: "New password", autoFocus: !0 })
              ] }),
              /* @__PURE__ */ e(to, { password: h })
            ] }),
            /* @__PURE__ */ s("div", { className: c.fieldWrap, children: [
              /* @__PURE__ */ e("span", { className: c.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
                /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
                /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
              ] }) }),
              /* @__PURE__ */ e("input", { className: c.fieldInput, type: "password", value: k, onChange: (R) => f(R.target.value), required: !0, autoComplete: "new-password", placeholder: "Confirm new password" })
            ] }),
            m && /* @__PURE__ */ e("p", { className: c.errorText, children: m }),
            /* @__PURE__ */ e(B, { type: "submit", loading: N, className: c.submitBtn, children: "Reset password" })
          ] })
        ] }),
        l === "success" && /* @__PURE__ */ s("div", { className: c.statusCenter, children: [
          /* @__PURE__ */ e("div", { className: c.successIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "48", height: "48", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
            /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
            /* @__PURE__ */ e("polyline", { points: "9 12 11 14 15 10" })
          ] }) }),
          /* @__PURE__ */ e("h1", { className: c.formTitle, children: "Password reset!" }),
          /* @__PURE__ */ e("p", { className: c.formSubtitle, children: "Your password has been updated. All previous sessions have been signed out. Sign in with your new password." }),
          /* @__PURE__ */ e(B, { className: c.actionBtn, onClick: () => o(b), children: "Sign in" })
        ] }),
        l === "error" && /* @__PURE__ */ s("div", { className: c.statusCenter, children: [
          /* @__PURE__ */ e("div", { className: c.errorIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "48", height: "48", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
            /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
            /* @__PURE__ */ e("line", { x1: "15", y1: "9", x2: "9", y2: "15" }),
            /* @__PURE__ */ e("line", { x1: "9", y1: "9", x2: "15", y2: "15" })
          ] }) }),
          /* @__PURE__ */ e("h1", { className: c.formTitle, children: "Reset failed" }),
          /* @__PURE__ */ e("p", { className: c.errorTextBlock, children: w }),
          /* @__PURE__ */ e(B, { className: c.actionBtn, onClick: () => o(I), children: "Request a new link" }),
          /* @__PURE__ */ e("p", { className: c.backLine, children: /* @__PURE__ */ e("button", { type: "button", className: c.backLink, onClick: () => o(b), children: "Back to sign in" }) })
        ] }),
        l === "no-token" && /* @__PURE__ */ s("div", { className: c.statusCenter, children: [
          /* @__PURE__ */ e("div", { className: c.errorIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "48", height: "48", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
            /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
            /* @__PURE__ */ e("line", { x1: "15", y1: "9", x2: "9", y2: "15" }),
            /* @__PURE__ */ e("line", { x1: "9", y1: "9", x2: "15", y2: "15" })
          ] }) }),
          /* @__PURE__ */ e("h1", { className: c.formTitle, children: "Invalid reset link" }),
          /* @__PURE__ */ e("p", { className: c.formSubtitle, children: "This link is invalid or has expired. Request a new password reset link from the login page." }),
          /* @__PURE__ */ e(B, { className: c.actionBtn, onClick: () => o(I), children: "Request a new link" }),
          /* @__PURE__ */ e("p", { className: c.backLine, children: /* @__PURE__ */ e("button", { type: "button", className: c.backLink, onClick: () => o(b), children: "Back to sign in" }) })
        ] })
      ] }),
      /* @__PURE__ */ e("p", { className: c.copyright, children: j })
    ] })
  ] });
}
const Ce = [
  { label: "At least 12 characters", test: (t) => t.length >= 12 },
  { label: "One uppercase letter", test: (t) => /[A-Z]/.test(t) },
  { label: "One lowercase letter", test: (t) => /[a-z]/.test(t) },
  { label: "One number", test: (t) => /[0-9]/.test(t) },
  { label: "One special character", test: (t) => /[^A-Za-z0-9]/.test(t) }
];
function oo({ password: t }) {
  return /* @__PURE__ */ e("ul", { className: c.checklist, "aria-label": "Password requirements", children: Ce.map(({ label: n, test: r }) => {
    const o = r(t);
    return /* @__PURE__ */ s("li", { className: c.checkItem, children: [
      /* @__PURE__ */ e(
        "svg",
        {
          className: `${c.checkIcon}${o ? ` ${c.met}` : ""}`,
          width: "14",
          height: "14",
          viewBox: "0 0 24 24",
          fill: "none",
          stroke: "currentColor",
          strokeWidth: "2.5",
          strokeLinecap: "round",
          strokeLinejoin: "round",
          "aria-hidden": "true",
          children: o ? /* @__PURE__ */ e("polyline", { points: "20 6 9 17 4 12" }) : /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "9" })
        }
      ),
      /* @__PURE__ */ e("span", { className: `${c.checkLabel}${o ? ` ${c.met}` : ""}`, children: n })
    ] }, n);
  }) });
}
function so(t) {
  for (const { label: n, test: r } of Ce)
    if (!r(t)) return `Password must include: ${n.toLowerCase()}.`;
  return null;
}
function io() {
  const { client: t, redirects: n, theme: r } = q(), o = X(), { accessToken: i, clearMustChangePassword: a } = A(), [l, d] = p(""), [h, u] = p(""), [k, f] = p(""), [m, C] = p(""), [w, S] = p(""), [N, y] = p(!1), [b, I] = p(!1), j = n.afterLogin ?? "/dashboard", z = r.copyright ?? "© 2026 Sentinel Auth. All rights reserved.", R = async ($) => {
    $.preventDefault(), C(""), S("");
    const W = so(h);
    if (W) {
      C(W);
      return;
    }
    if (h !== k) {
      C("Passwords do not match.");
      return;
    }
    y(!0);
    try {
      await t.user.changePassword(i, {
        current_password: l,
        new_password: h
      }), a(), I(!0);
    } catch (L) {
      const U = L instanceof D ? L.message : "Failed to change password. Please check your current password and try again.";
      S(U);
    } finally {
      y(!1);
    }
  };
  return /* @__PURE__ */ s("div", { className: c.page, children: [
    /* @__PURE__ */ e(G, { tagline: "Set a new password", taglineSubtext: "A new password is required to continue." }),
    /* @__PURE__ */ s("div", { className: c.formPanel, children: [
      /* @__PURE__ */ e("div", { className: c.formCard, children: b ? /* @__PURE__ */ s("div", { className: c.statusCenter, children: [
        /* @__PURE__ */ e("div", { className: c.successIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "48", height: "48", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
          /* @__PURE__ */ e("polyline", { points: "9 12 11 14 15 10" })
        ] }) }),
        /* @__PURE__ */ e("h1", { className: c.formTitle, children: "Password updated!" }),
        /* @__PURE__ */ e("p", { className: c.formSubtitle, children: "Your password has been changed. All previous sessions have been signed out." }),
        /* @__PURE__ */ e(B, { className: c.actionBtn, onClick: () => o(j), children: "Go to dashboard" })
      ] }) : /* @__PURE__ */ s(H, { children: [
        /* @__PURE__ */ s("div", { className: c.formHeader, children: [
          /* @__PURE__ */ e("div", { className: c.lockIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "40", height: "40", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.5", strokeLinecap: "round", strokeLinejoin: "round", children: [
            /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
            /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
          ] }) }),
          /* @__PURE__ */ e("h1", { className: c.formTitle, children: "Change your password" }),
          /* @__PURE__ */ e("p", { className: c.formSubtitle, children: "A temporary password was set for your account. Choose a new password to get started." })
        ] }),
        /* @__PURE__ */ s("form", { onSubmit: R, className: c.form, children: [
          /* @__PURE__ */ s("div", { className: c.fieldWrap, children: [
            /* @__PURE__ */ e("span", { className: c.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
              /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
            ] }) }),
            /* @__PURE__ */ e("input", { className: c.fieldInput, type: "password", value: l, onChange: ($) => d($.target.value), required: !0, autoComplete: "current-password", placeholder: "Temporary password", autoFocus: !0 })
          ] }),
          /* @__PURE__ */ s("div", { children: [
            /* @__PURE__ */ s("div", { className: c.fieldWrap, children: [
              /* @__PURE__ */ e("span", { className: c.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
                /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
                /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
              ] }) }),
              /* @__PURE__ */ e("input", { className: c.fieldInput, type: "password", value: h, onChange: ($) => u($.target.value), required: !0, autoComplete: "new-password", placeholder: "New password" })
            ] }),
            /* @__PURE__ */ e(oo, { password: h })
          ] }),
          /* @__PURE__ */ s("div", { className: c.fieldWrap, children: [
            /* @__PURE__ */ e("span", { className: c.fieldIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
              /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
            ] }) }),
            /* @__PURE__ */ e("input", { className: c.fieldInput, type: "password", value: k, onChange: ($) => f($.target.value), required: !0, autoComplete: "new-password", placeholder: "Confirm new password" })
          ] }),
          m && /* @__PURE__ */ e("p", { className: c.errorText, children: m }),
          w && /* @__PURE__ */ e("p", { className: c.errorText, children: w }),
          /* @__PURE__ */ e(B, { type: "submit", loading: N, className: c.submitBtn, children: "Set new password" })
        ] })
      ] }) }),
      /* @__PURE__ */ e("p", { className: c.copyright, children: z })
    ] })
  ] });
}
var Y = /* @__PURE__ */ ((t) => (t[t.Border = -1] = "Border", t[t.Data = 0] = "Data", t[t.Function = 1] = "Function", t[t.Position = 2] = "Position", t[t.Timing = 3] = "Timing", t[t.Alignment = 4] = "Alignment", t))(Y || {}), ao = Object.defineProperty, lo = (t, n, r) => n in t ? ao(t, n, { enumerable: !0, configurable: !0, writable: !0, value: r }) : t[n] = r, ne = (t, n, r) => (lo(t, typeof n != "symbol" ? n + "" : n, r), r);
const co = [0, 1], Ne = [1, 0], be = [2, 3], xe = [3, 2], ho = {
  L: co,
  M: Ne,
  Q: be,
  H: xe
}, uo = /^[0-9]*$/, mo = /^[A-Z0-9 $%*+.\/:-]*$/, ie = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:", ue = 1, me = 40, _e = 3, fo = 3, re = 40, po = 10, Le = [
  // Version: (note that index 0 is for padding, and is set to an illegal value)
  // 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40    Error correction level
  [-1, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
  // Low
  [-1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28],
  // Medium
  [-1, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
  // Quartile
  [-1, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30]
  // High
], Pe = [
  // Version: (note that index 0 is for padding, and is set to an illegal value)
  // 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40    Error correction level
  [-1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25],
  // Low
  [-1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49],
  // Medium
  [-1, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68],
  // Quartile
  [-1, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81]
  // High
];
class go {
  /* -- Constructor (low level) and fields -- */
  // Creates a new QR Code with the given version number,
  // error correction level, data codeword bytes, and mask number.
  // This is a low-level API that most users should not use directly.
  // A mid-level API is the encodeSegments() function.
  constructor(n, r, o, i) {
    if (this.version = n, this.ecc = r, ne(this, "size"), ne(this, "mask"), ne(this, "modules", []), ne(this, "types", []), n < ue || n > me)
      throw new RangeError("Version value out of range");
    if (i < -1 || i > 7)
      throw new RangeError("Mask value out of range");
    this.size = n * 4 + 17;
    const a = Array.from({ length: this.size }, () => !1);
    for (let d = 0; d < this.size; d++)
      this.modules.push(a.slice()), this.types.push(a.map(() => 0));
    this.drawFunctionPatterns();
    const l = this.addEccAndInterleave(o);
    if (this.drawCodewords(l), i === -1) {
      let d = 1e9;
      for (let h = 0; h < 8; h++) {
        this.applyMask(h), this.drawFormatBits(h);
        const u = this.getPenaltyScore();
        u < d && (i = h, d = u), this.applyMask(h);
      }
    }
    this.mask = i, this.applyMask(i), this.drawFormatBits(i);
  }
  /* -- Accessor methods -- */
  // Returns the color of the module (pixel) at the given coordinates, which is false
  // for light or true for dark. The top left corner has the coordinates (x=0, y=0).
  // If the given coordinates are out of bounds, then false (light) is returned.
  getModule(n, r) {
    return n >= 0 && n < this.size && r >= 0 && r < this.size && this.modules[r][n];
  }
  /* -- Private helper methods for constructor: Drawing function modules -- */
  // Reads this object's version field, and draws and marks all function modules.
  drawFunctionPatterns() {
    for (let o = 0; o < this.size; o++)
      this.setFunctionModule(6, o, o % 2 === 0, Y.Timing), this.setFunctionModule(o, 6, o % 2 === 0, Y.Timing);
    this.drawFinderPattern(3, 3), this.drawFinderPattern(this.size - 4, 3), this.drawFinderPattern(3, this.size - 4);
    const n = this.getAlignmentPatternPositions(), r = n.length;
    for (let o = 0; o < r; o++)
      for (let i = 0; i < r; i++)
        o === 0 && i === 0 || o === 0 && i === r - 1 || o === r - 1 && i === 0 || this.drawAlignmentPattern(n[o], n[i]);
    this.drawFormatBits(0), this.drawVersion();
  }
  // Draws two copies of the format bits (with its own error correction code)
  // based on the given mask and this object's error correction level field.
  drawFormatBits(n) {
    const r = this.ecc[1] << 3 | n;
    let o = r;
    for (let a = 0; a < 10; a++)
      o = o << 1 ^ (o >>> 9) * 1335;
    const i = (r << 10 | o) ^ 21522;
    for (let a = 0; a <= 5; a++)
      this.setFunctionModule(8, a, O(i, a));
    this.setFunctionModule(8, 7, O(i, 6)), this.setFunctionModule(8, 8, O(i, 7)), this.setFunctionModule(7, 8, O(i, 8));
    for (let a = 9; a < 15; a++)
      this.setFunctionModule(14 - a, 8, O(i, a));
    for (let a = 0; a < 8; a++)
      this.setFunctionModule(this.size - 1 - a, 8, O(i, a));
    for (let a = 8; a < 15; a++)
      this.setFunctionModule(8, this.size - 15 + a, O(i, a));
    this.setFunctionModule(8, this.size - 8, !0);
  }
  // Draws two copies of the version bits (with its own error correction code),
  // based on this object's version field, iff 7 <= version <= 40.
  drawVersion() {
    if (this.version < 7)
      return;
    let n = this.version;
    for (let o = 0; o < 12; o++)
      n = n << 1 ^ (n >>> 11) * 7973;
    const r = this.version << 12 | n;
    for (let o = 0; o < 18; o++) {
      const i = O(r, o), a = this.size - 11 + o % 3, l = Math.floor(o / 3);
      this.setFunctionModule(a, l, i), this.setFunctionModule(l, a, i);
    }
  }
  // Draws a 9*9 finder pattern including the border separator,
  // with the center module at (x, y). Modules can be out of bounds.
  drawFinderPattern(n, r) {
    for (let o = -4; o <= 4; o++)
      for (let i = -4; i <= 4; i++) {
        const a = Math.max(Math.abs(i), Math.abs(o)), l = n + i, d = r + o;
        l >= 0 && l < this.size && d >= 0 && d < this.size && this.setFunctionModule(l, d, a !== 2 && a !== 4, Y.Position);
      }
  }
  // Draws a 5*5 alignment pattern, with the center module
  // at (x, y). All modules must be in bounds.
  drawAlignmentPattern(n, r) {
    for (let o = -2; o <= 2; o++)
      for (let i = -2; i <= 2; i++)
        this.setFunctionModule(
          n + i,
          r + o,
          Math.max(Math.abs(i), Math.abs(o)) !== 1,
          Y.Alignment
        );
  }
  // Sets the color of a module and marks it as a function module.
  // Only used by the constructor. Coordinates must be in bounds.
  setFunctionModule(n, r, o, i = Y.Function) {
    this.modules[r][n] = o, this.types[r][n] = i;
  }
  /* -- Private helper methods for constructor: Codewords and masking -- */
  // Returns a new byte string representing the given data with the appropriate error correction
  // codewords appended to it, based on this object's version and error correction level.
  addEccAndInterleave(n) {
    const r = this.version, o = this.ecc;
    if (n.length !== oe(r, o))
      throw new RangeError("Invalid argument");
    const i = Pe[o[0]][r], a = Le[o[0]][r], l = Math.floor(ce(r) / 8), d = i - l % i, h = Math.floor(l / i), u = [], k = xo(a);
    for (let m = 0, C = 0; m < i; m++) {
      const w = n.slice(C, C + h - a + (m < d ? 0 : 1));
      C += w.length;
      const S = Lo(w, k);
      m < d && w.push(0), u.push(w.concat(S));
    }
    const f = [];
    for (let m = 0; m < u[0].length; m++)
      u.forEach((C, w) => {
        (m !== h - a || w >= d) && f.push(C[m]);
      });
    return f;
  }
  // Draws the given sequence of 8-bit codewords (data and error correction) onto the entire
  // data area of this QR Code. Function modules need to be marked off before this is called.
  drawCodewords(n) {
    if (n.length !== Math.floor(ce(this.version) / 8))
      throw new RangeError("Invalid argument");
    let r = 0;
    for (let o = this.size - 1; o >= 1; o -= 2) {
      o === 6 && (o = 5);
      for (let i = 0; i < this.size; i++)
        for (let a = 0; a < 2; a++) {
          const l = o - a, h = (o + 1 & 2) === 0 ? this.size - 1 - i : i;
          !this.types[h][l] && r < n.length * 8 && (this.modules[h][l] = O(n[r >>> 3], 7 - (r & 7)), r++);
        }
    }
  }
  // XORs the codeword modules in this QR Code with the given mask pattern.
  // The function modules must be marked and the codeword bits must be drawn
  // before masking. Due to the arithmetic of XOR, calling applyMask() with
  // the same mask value a second time will undo the mask. A final well-formed
  // QR Code needs exactly one (not zero, two, etc.) mask applied.
  applyMask(n) {
    if (n < 0 || n > 7)
      throw new RangeError("Mask value out of range");
    for (let r = 0; r < this.size; r++)
      for (let o = 0; o < this.size; o++) {
        let i;
        switch (n) {
          case 0:
            i = (o + r) % 2 === 0;
            break;
          case 1:
            i = r % 2 === 0;
            break;
          case 2:
            i = o % 3 === 0;
            break;
          case 3:
            i = (o + r) % 3 === 0;
            break;
          case 4:
            i = (Math.floor(o / 3) + Math.floor(r / 2)) % 2 === 0;
            break;
          case 5:
            i = o * r % 2 + o * r % 3 === 0;
            break;
          case 6:
            i = (o * r % 2 + o * r % 3) % 2 === 0;
            break;
          case 7:
            i = ((o + r) % 2 + o * r % 3) % 2 === 0;
            break;
          default:
            throw new Error("Unreachable");
        }
        !this.types[r][o] && i && (this.modules[r][o] = !this.modules[r][o]);
      }
  }
  // Calculates and returns the penalty score based on state of this QR Code's current modules.
  // This is used by the automatic mask choice algorithm to find the mask pattern that yields the lowest score.
  getPenaltyScore() {
    let n = 0;
    for (let a = 0; a < this.size; a++) {
      let l = !1, d = 0;
      const h = [0, 0, 0, 0, 0, 0, 0];
      for (let u = 0; u < this.size; u++)
        this.modules[a][u] === l ? (d++, d === 5 ? n += _e : d > 5 && n++) : (this.finderPenaltyAddHistory(d, h), l || (n += this.finderPenaltyCountPatterns(h) * re), l = this.modules[a][u], d = 1);
      n += this.finderPenaltyTerminateAndCount(l, d, h) * re;
    }
    for (let a = 0; a < this.size; a++) {
      let l = !1, d = 0;
      const h = [0, 0, 0, 0, 0, 0, 0];
      for (let u = 0; u < this.size; u++)
        this.modules[u][a] === l ? (d++, d === 5 ? n += _e : d > 5 && n++) : (this.finderPenaltyAddHistory(d, h), l || (n += this.finderPenaltyCountPatterns(h) * re), l = this.modules[u][a], d = 1);
      n += this.finderPenaltyTerminateAndCount(l, d, h) * re;
    }
    for (let a = 0; a < this.size - 1; a++)
      for (let l = 0; l < this.size - 1; l++) {
        const d = this.modules[a][l];
        d === this.modules[a][l + 1] && d === this.modules[a + 1][l] && d === this.modules[a + 1][l + 1] && (n += fo);
      }
    let r = 0;
    for (const a of this.modules)
      r = a.reduce((l, d) => l + (d ? 1 : 0), r);
    const o = this.size * this.size, i = Math.ceil(Math.abs(r * 20 - o * 10) / o) - 1;
    return n += i * po, n;
  }
  /* -- Private helper functions -- */
  // Returns an ascending list of positions of alignment patterns for this version number.
  // Each position is in the range [0,177), and are used on both the x and y axes.
  // This could be implemented as lookup table of 40 variable-length lists of integers.
  getAlignmentPatternPositions() {
    if (this.version === 1)
      return [];
    {
      const n = Math.floor(this.version / 7) + 2, r = this.version === 32 ? 26 : Math.ceil((this.version * 4 + 4) / (n * 2 - 2)) * 2, o = [6];
      for (let i = this.size - 7; o.length < n; i -= r)
        o.splice(1, 0, i);
      return o;
    }
  }
  // Can only be called immediately after a light run is added, and
  // returns either 0, 1, or 2. A helper function for getPenaltyScore().
  finderPenaltyCountPatterns(n) {
    const r = n[1], o = r > 0 && n[2] === r && n[3] === r * 3 && n[4] === r && n[5] === r;
    return (o && n[0] >= r * 4 && n[6] >= r ? 1 : 0) + (o && n[6] >= r * 4 && n[0] >= r ? 1 : 0);
  }
  // Must be called at the end of a line (row or column) of modules. A helper function for getPenaltyScore().
  finderPenaltyTerminateAndCount(n, r, o) {
    return n && (this.finderPenaltyAddHistory(r, o), r = 0), r += this.size, this.finderPenaltyAddHistory(r, o), this.finderPenaltyCountPatterns(o);
  }
  // Pushes the given value to the front and drops the last value. A helper function for getPenaltyScore().
  finderPenaltyAddHistory(n, r) {
    r[0] === 0 && (n += this.size), r.pop(), r.unshift(n);
  }
}
function V(t, n, r) {
  if (n < 0 || n > 31 || t >>> n)
    throw new RangeError("Value out of range");
  for (let o = n - 1; o >= 0; o--)
    r.push(t >>> o & 1);
}
function O(t, n) {
  return (t >>> n & 1) !== 0;
}
class fe {
  // Creates a new QR Code segment with the given attributes and data.
  // The character count (numChars) must agree with the mode and the bit buffer length,
  // but the constraint isn't checked. The given bit buffer is cloned and stored.
  constructor(n, r, o) {
    if (this.mode = n, this.numChars = r, this.bitData = o, r < 0)
      throw new RangeError("Invalid argument");
    this.bitData = o.slice();
  }
  /* -- Methods -- */
  // Returns a new copy of the data bits of this segment.
  getData() {
    return this.bitData.slice();
  }
}
const _o = [1, 10, 12, 14], ko = [2, 9, 11, 13], vo = [4, 8, 16, 16];
function Se(t, n) {
  return t[Math.floor((n + 7) / 17) + 1];
}
function Ie(t) {
  const n = [];
  for (const r of t)
    V(r, 8, n);
  return new fe(vo, t.length, n);
}
function wo(t) {
  if (!Be(t))
    throw new RangeError("String contains non-numeric characters");
  const n = [];
  for (let r = 0; r < t.length; ) {
    const o = Math.min(t.length - r, 3);
    V(Number.parseInt(t.substring(r, r + o), 10), o * 3 + 1, n), r += o;
  }
  return new fe(_o, t.length, n);
}
function yo(t) {
  if (!$e(t))
    throw new RangeError("String contains unencodable characters in alphanumeric mode");
  const n = [];
  let r;
  for (r = 0; r + 2 <= t.length; r += 2) {
    let o = ie.indexOf(t.charAt(r)) * 45;
    o += ie.indexOf(t.charAt(r + 1)), V(o, 11, n);
  }
  return r < t.length && V(ie.indexOf(t.charAt(r)), 6, n), new fe(ko, t.length, n);
}
function Co(t) {
  return t === "" ? [] : Be(t) ? [wo(t)] : $e(t) ? [yo(t)] : [Ie(bo(t))];
}
function Be(t) {
  return uo.test(t);
}
function $e(t) {
  return mo.test(t);
}
function No(t, n) {
  let r = 0;
  for (const o of t) {
    const i = Se(o.mode, n);
    if (o.numChars >= 1 << i)
      return Number.POSITIVE_INFINITY;
    r += 4 + i + o.bitData.length;
  }
  return r;
}
function bo(t) {
  t = encodeURI(t);
  const n = [];
  for (let r = 0; r < t.length; r++)
    t.charAt(r) !== "%" ? n.push(t.charCodeAt(r)) : (n.push(Number.parseInt(t.substring(r + 1, r + 3), 16)), r += 2);
  return n;
}
function ce(t) {
  if (t < ue || t > me)
    throw new RangeError("Version number out of range");
  let n = (16 * t + 128) * t + 64;
  if (t >= 2) {
    const r = Math.floor(t / 7) + 2;
    n -= (25 * r - 10) * r - 55, t >= 7 && (n -= 36);
  }
  return n;
}
function oe(t, n) {
  return Math.floor(ce(t) / 8) - Le[n[0]][t] * Pe[n[0]][t];
}
function xo(t) {
  if (t < 1 || t > 255)
    throw new RangeError("Degree out of range");
  const n = [];
  for (let o = 0; o < t - 1; o++)
    n.push(0);
  n.push(1);
  let r = 1;
  for (let o = 0; o < t; o++) {
    for (let i = 0; i < n.length; i++)
      n[i] = de(n[i], r), i + 1 < n.length && (n[i] ^= n[i + 1]);
    r = de(r, 2);
  }
  return n;
}
function Lo(t, n) {
  const r = n.map((o) => 0);
  for (const o of t) {
    const i = o ^ r.shift();
    r.push(0), n.forEach((a, l) => r[l] ^= de(a, i));
  }
  return r;
}
function de(t, n) {
  if (t >>> 8 || n >>> 8)
    throw new RangeError("Byte out of range");
  let r = 0;
  for (let o = 7; o >= 0; o--)
    r = r << 1 ^ (r >>> 7) * 285, r ^= (n >>> o & 1) * t;
  return r;
}
function Po(t, n, r = 1, o = 40, i = -1, a = !0) {
  if (!(ue <= r && r <= o && o <= me) || i < -1 || i > 7)
    throw new RangeError("Invalid value");
  let l, d;
  for (l = r; ; l++) {
    const f = oe(l, n) * 8, m = No(t, l);
    if (m <= f) {
      d = m;
      break;
    }
    if (l >= o)
      throw new RangeError("Data too long");
  }
  for (const f of [Ne, be, xe])
    a && d <= oe(l, f) * 8 && (n = f);
  const h = [];
  for (const f of t) {
    V(f.mode[0], 4, h), V(f.numChars, Se(f.mode, l), h);
    for (const m of f.getData())
      h.push(m);
  }
  const u = oe(l, n) * 8;
  V(0, Math.min(4, u - h.length), h), V(0, (8 - h.length % 8) % 8, h);
  for (let f = 236; h.length < u; f ^= 253)
    V(f, 8, h);
  const k = Array.from({ length: Math.ceil(h.length / 8) }, () => 0);
  return h.forEach((f, m) => k[m >>> 3] |= f << 7 - (m & 7)), new go(l, n, k, i);
}
function So(t, n) {
  const {
    ecc: r = "L",
    boostEcc: o = !1,
    minVersion: i = 1,
    maxVersion: a = 40,
    maskPattern: l = -1,
    border: d = 1
  } = n || {}, h = typeof t == "string" ? Co(t) : Array.isArray(t) ? [Ie(t)] : void 0;
  if (!h)
    throw new Error(`uqr only supports encoding string and binary data, but got: ${typeof t}`);
  const u = Po(
    h,
    ho[r],
    i,
    a,
    l,
    o
  ), k = Io({
    version: u.version,
    maskPattern: u.mask,
    size: u.size,
    data: u.modules,
    types: u.types
  }, d);
  return n?.invert && (k.data = k.data.map((f) => f.map((m) => !m))), n?.onEncoded?.(k), k;
}
function Io(t, n = 1) {
  if (!n)
    return t;
  const { size: r } = t, o = r + n * 2;
  t.size = o, t.data.forEach((a) => {
    for (let l = 0; l < n; l++)
      a.unshift(!1), a.push(!1);
  });
  for (let a = 0; a < n; a++)
    t.data.unshift(Array.from({ length: o }, (l) => !1)), t.data.push(Array.from({ length: o }, (l) => !1));
  const i = Y.Border;
  t.types.forEach((a) => {
    for (let l = 0; l < n; l++)
      a.unshift(i), a.push(i);
  });
  for (let a = 0; a < n; a++)
    t.types.unshift(Array.from({ length: o }, (l) => i)), t.types.push(Array.from({ length: o }, (l) => i));
  return t;
}
function Bo(t, n = {}) {
  const r = So(t, n), {
    pixelSize: o = 10,
    whiteColor: i = "white",
    blackColor: a = "black"
  } = n, l = r.size * o, d = r.size * o;
  let h = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${d} ${l}">`;
  const u = [];
  for (let k = 0; k < r.size; k++)
    for (let f = 0; f < r.size; f++) {
      const m = f * o, C = k * o;
      r.data[k][f] && u.push(`M${m},${C}h${o}v${o}h-${o}z`);
    }
  return h += `<rect fill="${i}" width="${d}" height="${l}"/>`, h += `<path fill="${a}" d="${u.join("")}"/>`, h += "</svg>", h;
}
function $o(t) {
  try {
    return new URL(t).searchParams.get("secret");
  } catch {
    return null;
  }
}
function Mo() {
  const { client: t, redirects: n, theme: r } = q(), o = X(), { accessToken: i, clearMfaSetupRequired: a } = A(), [l, d] = p("intro"), [h, u] = p(null), [k, f] = p([]), [m, C] = p(""), [w, S] = p(""), [N, y] = p(!1), [b, I] = p(!1), [j, z] = p(!1), R = n.afterLogin ?? "/dashboard", $ = r.copyright ?? "© 2026 Sentinel Auth. All rights reserved.", W = async () => {
    S(""), y(!0);
    try {
      const { otpauth_uri: M } = await t.mfa.totpStart(i);
      u(M), d("confirm");
    } catch (M) {
      S(M instanceof D ? M.message : "Failed to start MFA setup.");
    } finally {
      y(!1);
    }
  }, L = async (M) => {
    M.preventDefault(), S(""), y(!0);
    try {
      const { recovery_codes: T } = await t.mfa.totpConfirm(i, { code: m });
      f(T), d("done");
    } catch (T) {
      S(T instanceof D ? T.message : "Invalid code. Please try again.");
    } finally {
      y(!1);
    }
  }, U = () => {
    a(), o(R);
  }, se = () => {
    navigator.clipboard.writeText(k.join(`
`)).then(() => {
      I(!0), setTimeout(() => I(!1), 2e3);
    });
  };
  return /* @__PURE__ */ s("div", { className: c.page, children: [
    /* @__PURE__ */ e(G, { tagline: "Set up two-factor authentication", taglineSubtext: "MFA is required for your account." }),
    /* @__PURE__ */ s("div", { className: c.formPanel, children: [
      /* @__PURE__ */ s("div", { className: c.formCard, children: [
        l === "intro" && /* @__PURE__ */ s(H, { children: [
          /* @__PURE__ */ s("div", { className: c.formHeader, children: [
            /* @__PURE__ */ e("div", { className: c.lockIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "40", height: "40", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.5", strokeLinecap: "round", strokeLinejoin: "round", children: [
              /* @__PURE__ */ e("rect", { x: "3", y: "11", width: "18", height: "11", rx: "2" }),
              /* @__PURE__ */ e("path", { d: "M7 11V7a5 5 0 0110 0v4" })
            ] }) }),
            /* @__PURE__ */ e("h1", { className: c.formTitle, children: "Set up MFA" }),
            /* @__PURE__ */ e("p", { className: c.formSubtitle, children: "Your administrator requires multi-factor authentication. Link an authenticator app to continue." })
          ] }),
          w && /* @__PURE__ */ e("p", { className: c.errorText, children: w }),
          /* @__PURE__ */ e(B, { className: c.submitBtn, onClick: W, loading: N, children: "Set up authenticator app" })
        ] }),
        l === "confirm" && /* @__PURE__ */ s(H, { children: [
          /* @__PURE__ */ s("div", { className: c.formHeader, children: [
            /* @__PURE__ */ e("h1", { className: c.formTitle, children: "Scan QR code" }),
            /* @__PURE__ */ e("p", { className: c.formSubtitle, children: "Scan this code with your authenticator app, then enter the 6-digit code to verify." })
          ] }),
          h && /* @__PURE__ */ e(
            "div",
            {
              className: c.qrWrap,
              dangerouslySetInnerHTML: { __html: Bo(h) },
              "aria-label": "TOTP QR code"
            }
          ),
          h && (() => {
            const M = $o(h);
            return M ? /* @__PURE__ */ s("details", { style: { marginBottom: "1rem" }, children: [
              /* @__PURE__ */ e("summary", { style: { cursor: "pointer", fontSize: "0.85rem", color: "var(--color-text-muted, #888)" }, children: "Can't scan? Enter the key manually" }),
              /* @__PURE__ */ s("div", { style: { display: "flex", gap: "0.5rem", alignItems: "center", marginTop: "0.5rem" }, children: [
                /* @__PURE__ */ e("code", { style: { fontSize: "0.8rem", wordBreak: "break-all" }, children: M }),
                /* @__PURE__ */ e(
                  "button",
                  {
                    type: "button",
                    style: { fontSize: "0.75rem", cursor: "pointer" },
                    onClick: () => {
                      navigator.clipboard.writeText(M).then(() => {
                        z(!0), setTimeout(() => z(!1), 2e3);
                      });
                    },
                    children: j ? "Copied!" : "Copy key"
                  }
                )
              ] })
            ] }) : null;
          })(),
          /* @__PURE__ */ s("form", { onSubmit: L, className: c.form, children: [
            /* @__PURE__ */ e("div", { className: c.fieldWrap, children: /* @__PURE__ */ e(
              "input",
              {
                className: c.fieldInput,
                type: "text",
                inputMode: "numeric",
                pattern: "[0-9]{6}",
                maxLength: 6,
                value: m,
                onChange: (M) => C(M.target.value.replace(/\D/g, "")),
                placeholder: "6-digit code",
                required: !0,
                autoFocus: !0,
                autoComplete: "one-time-code"
              }
            ) }),
            w && /* @__PURE__ */ e("p", { className: c.errorText, children: w }),
            /* @__PURE__ */ e(B, { type: "submit", loading: N, className: c.submitBtn, disabled: m.length !== 6, children: "Verify and enable MFA" })
          ] })
        ] }),
        l === "done" && /* @__PURE__ */ s("div", { className: c.statusCenter, children: [
          /* @__PURE__ */ e("div", { className: c.successIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "48", height: "48", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
            /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
            /* @__PURE__ */ e("polyline", { points: "9 12 11 14 15 10" })
          ] }) }),
          /* @__PURE__ */ e("h1", { className: c.formTitle, children: "MFA enabled!" }),
          /* @__PURE__ */ e("p", { className: c.formSubtitle, children: "Save these recovery codes somewhere safe. Each code can only be used once." }),
          /* @__PURE__ */ e("ul", { style: { listStyle: "none", padding: 0, margin: "1rem 0", display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.4rem" }, children: k.map((M) => /* @__PURE__ */ e("li", { children: /* @__PURE__ */ e("code", { style: { fontSize: "0.85rem" }, children: M }) }, M)) }),
          /* @__PURE__ */ e(B, { className: c.actionBtn, onClick: se, style: { marginBottom: "0.75rem" }, children: b ? "Copied!" : "Copy all codes" }),
          /* @__PURE__ */ e(B, { className: c.actionBtn, onClick: U, children: "Go to dashboard" })
        ] })
      ] }),
      /* @__PURE__ */ e("p", { className: c.copyright, children: $ })
    ] })
  ] });
}
const Ao = "_page_1bvyy_2", To = "_lockSvg_1bvyy_72", jo = "_formPanel_1bvyy_107", Eo = "_topControls_1bvyy_117", qo = "_topControlBtn_1bvyy_126", Ro = "_topControlChevron_1bvyy_138", Wo = "_formCard_1bvyy_143", zo = "_statusCenter_1bvyy_153", Fo = "_errorIcon_1bvyy_161", Oo = "_formTitle_1bvyy_173", Vo = "_formSubtitle_1bvyy_181", Ho = "_actionBtn_1bvyy_189", Do = "_copyright_1bvyy_195", E = {
  page: Ao,
  lockSvg: To,
  formPanel: jo,
  topControls: Eo,
  topControlBtn: qo,
  topControlChevron: Ro,
  formCard: Wo,
  statusCenter: zo,
  errorIcon: Fo,
  formTitle: Oo,
  formSubtitle: Vo,
  actionBtn: Ho,
  copyright: Do
};
function Uo() {
  return /* @__PURE__ */ s(
    "svg",
    {
      className: E.lockSvg,
      viewBox: "0 0 120 140",
      fill: "none",
      xmlns: "http://www.w3.org/2000/svg",
      "aria-hidden": "true",
      children: [
        /* @__PURE__ */ s("defs", { children: [
          /* @__PURE__ */ s("linearGradient", { id: "unauthShieldGrad", x1: "0%", y1: "0%", x2: "100%", y2: "100%", children: [
            /* @__PURE__ */ e("stop", { offset: "0%", stopColor: "#f87171" }),
            /* @__PURE__ */ e("stop", { offset: "100%", stopColor: "#ef4444" })
          ] }),
          /* @__PURE__ */ s("linearGradient", { id: "unauthShieldInner", x1: "0%", y1: "0%", x2: "100%", y2: "100%", children: [
            /* @__PURE__ */ e("stop", { offset: "0%", stopColor: "rgba(248,113,113,0.15)" }),
            /* @__PURE__ */ e("stop", { offset: "100%", stopColor: "rgba(239,68,68,0.08)" })
          ] })
        ] }),
        /* @__PURE__ */ e(
          "path",
          {
            d: "M60 4L8 26v42c0 31.4 22.1 60.8 52 68 29.9-7.2 52-36.6 52-68V26L60 4z",
            fill: "url(#unauthShieldInner)",
            stroke: "url(#unauthShieldGrad)",
            strokeWidth: "2"
          }
        ),
        /* @__PURE__ */ e(
          "path",
          {
            d: "M60 18L22 36v32c0 22.8 16.2 44.1 38 49.4C81.8 112.1 98 90.8 98 68V36L60 18z",
            fill: "url(#unauthShieldInner)",
            stroke: "url(#unauthShieldGrad)",
            strokeWidth: "1",
            strokeOpacity: "0.5"
          }
        ),
        /* @__PURE__ */ e("rect", { x: "46", y: "66", width: "28", height: "22", rx: "4", fill: "url(#unauthShieldGrad)" }),
        /* @__PURE__ */ e(
          "path",
          {
            d: "M50 66v-6a10 10 0 0120 0v6",
            stroke: "url(#unauthShieldGrad)",
            strokeWidth: "3.5",
            strokeLinecap: "round",
            fill: "none"
          }
        ),
        /* @__PURE__ */ e("line", { x1: "53", y1: "72", x2: "67", y2: "82", stroke: "#070d1a", strokeWidth: "2.5", strokeLinecap: "round" }),
        /* @__PURE__ */ e("line", { x1: "67", y1: "72", x2: "53", y2: "82", stroke: "#070d1a", strokeWidth: "2.5", strokeLinecap: "round" })
      ]
    }
  );
}
function Go() {
  const { client: t, redirects: n, theme: r } = q(), { userId: o, clearTokens: i } = A(), a = n.afterLogout ?? "/login", l = r.copyright ?? "© 2026 Sentinel Auth. All rights reserved.", d = J(async () => {
    try {
      o && await t.logout(o);
    } finally {
      i(), window.location.href = a;
    }
  }, [o, i, t, a]);
  return /* @__PURE__ */ s("div", { className: E.page, children: [
    /* @__PURE__ */ e(
      G,
      {
        tagline: "Access control",
        taglineSubtext: "Your identity is verified, but access was denied.",
        defaultIcon: /* @__PURE__ */ e(Uo, {}),
        showOrbits: !1
      }
    ),
    /* @__PURE__ */ s("div", { className: E.formPanel, children: [
      /* @__PURE__ */ s("div", { className: E.topControls, "aria-hidden": "true", children: [
        /* @__PURE__ */ e("span", { className: E.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
          /* @__PURE__ */ e("line", { x1: "2", y1: "12", x2: "22", y2: "12" }),
          /* @__PURE__ */ e("path", { d: "M12 2a15.3 15.3 0 010 20M12 2a15.3 15.3 0 000 20" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: E.topControlBtn, children: /* @__PURE__ */ s("svg", { width: "15", height: "15", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("line", { x1: "3", y1: "6", x2: "21", y2: "6" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "12", x2: "21", y2: "12" }),
          /* @__PURE__ */ e("line", { x1: "3", y1: "18", x2: "21", y2: "18" })
        ] }) }),
        /* @__PURE__ */ e("span", { className: E.topControlChevron, children: /* @__PURE__ */ e("svg", { width: "12", height: "12", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2", strokeLinecap: "round", strokeLinejoin: "round", children: /* @__PURE__ */ e("polyline", { points: "6 9 12 15 18 9" }) }) })
      ] }),
      /* @__PURE__ */ e("div", { className: E.formCard, children: /* @__PURE__ */ s("div", { className: E.statusCenter, children: [
        /* @__PURE__ */ e("div", { className: E.errorIcon, "aria-hidden": "true", children: /* @__PURE__ */ s("svg", { width: "48", height: "48", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "1.8", strokeLinecap: "round", strokeLinejoin: "round", children: [
          /* @__PURE__ */ e("circle", { cx: "12", cy: "12", r: "10" }),
          /* @__PURE__ */ e("line", { x1: "15", y1: "9", x2: "9", y2: "15" }),
          /* @__PURE__ */ e("line", { x1: "9", y1: "9", x2: "15", y2: "15" })
        ] }) }),
        /* @__PURE__ */ e("h1", { className: E.formTitle, children: "Access denied" }),
        /* @__PURE__ */ e("p", { className: E.formSubtitle, children: "Your account doesn't have permission to access this area. Contact your administrator if you believe this is a mistake." }),
        /* @__PURE__ */ e(B, { className: E.actionBtn, onClick: d, children: "Sign out" })
      ] }) }),
      /* @__PURE__ */ e("p", { className: E.copyright, children: l })
    ] })
  ] });
}
function os() {
  const { redirects: t } = q(), n = t.login ?? "/login", r = t.register ?? "/register", o = t.verifyEmail ?? "/verify-email", i = t.forgotPassword ?? "/forgot-password", a = "/reset-password", l = t.changePassword ?? "/change-password", d = t.setupMfa ?? "/setup-mfa", h = t.unauthorized ?? "/unauthorized";
  return /* @__PURE__ */ s(Ve, { children: [
    /* @__PURE__ */ s(F, { element: /* @__PURE__ */ e(Ze, {}), children: [
      /* @__PURE__ */ e(F, { path: n, element: /* @__PURE__ */ e(Zt, {}) }),
      /* @__PURE__ */ e(F, { path: r, element: /* @__PURE__ */ e(Nn, {}) })
    ] }),
    /* @__PURE__ */ e(F, { path: o, element: /* @__PURE__ */ e(er, {}) }),
    /* @__PURE__ */ e(F, { path: i, element: /* @__PURE__ */ e(xr, {}) }),
    /* @__PURE__ */ e(F, { path: a, element: /* @__PURE__ */ e(ro, {}) }),
    /* @__PURE__ */ s(F, { element: /* @__PURE__ */ e(Ye, {}), children: [
      /* @__PURE__ */ e(F, { path: l, element: /* @__PURE__ */ e(io, {}) }),
      /* @__PURE__ */ e(F, { path: d, element: /* @__PURE__ */ e(Mo, {}) }),
      /* @__PURE__ */ e(F, { path: h, element: /* @__PURE__ */ e(Go, {}) })
    ] })
  ] });
}
export {
  rs as AuthorizedRoute,
  G as BrandPanel,
  B as Button,
  io as ChangePasswordForcedPage,
  xr as ForgotPasswordPage,
  Zt as LoginPage,
  Ye as ProtectedRoute,
  Ze as PublicRoute,
  Nn as RegisterPage,
  ro as ResetPasswordPage,
  we as SentinelAuthContext,
  ts as SentinelAuthProvider,
  os as SentinelAuthRoutes,
  Mo as SetupMfaForcedPage,
  Go as UnauthorizedPage,
  er as VerifyEmailPage,
  ns as createSentinelQueryClient,
  De as refreshTokens,
  He as registerTokenRefreshClient,
  Ge as useAuth,
  A as useAuthStore,
  q as useSentinelAuth,
  q as useSentinelConfig
};
