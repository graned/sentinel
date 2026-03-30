interface AuthState {
    userId: string | null;
    accessToken: string | null;
    refreshToken: string | null;
    isAuthenticated: boolean;
    emailVerified: boolean;
    isAdmin: boolean;
    mustChangePassword: boolean;
    mfaSetupRequired: boolean;
    userEmail: string | null;
    firstName: string | null;
    lastName: string | null;
    setSession: (userId: string, access: string, refresh: string, emailVerified: boolean, mustChangePassword?: boolean) => void;
    setIsAdmin: (isAdmin: boolean) => void;
    setUserProfile: (email: string, firstName: string | null, lastName: string | null) => void;
    clearMustChangePassword: () => void;
    setMfaSetupRequired: (val: boolean) => void;
    clearMfaSetupRequired: () => void;
    clearTokens: () => void;
}
export declare const useAuthStore: import("zustand").UseBoundStore<Omit<import("zustand").StoreApi<AuthState>, "setState" | "persist"> & {
    setState(partial: AuthState | Partial<AuthState> | ((state: AuthState) => AuthState | Partial<AuthState>), replace?: false | undefined): unknown;
    setState(state: AuthState | ((state: AuthState) => AuthState), replace: true): unknown;
    persist: {
        setOptions: (options: Partial<import("zustand/middleware").PersistOptions<AuthState, AuthState, unknown>>) => void;
        clearStorage: () => void;
        rehydrate: () => Promise<void> | void;
        hasHydrated: () => boolean;
        onHydrate: (fn: (state: AuthState) => void) => () => void;
        onFinishHydration: (fn: (state: AuthState) => void) => () => void;
        getOptions: () => Partial<import("zustand/middleware").PersistOptions<AuthState, AuthState, unknown>>;
    };
}>;
export {};
//# sourceMappingURL=authStore.d.ts.map