import type { LoginRequest } from "@sentinel/auth-sdk";
export declare function useAuth(): {
    isAuthenticated: boolean;
    isLoading: boolean;
    error: string | null;
    login: (data: LoginRequest) => Promise<{
        success: true;
        mfa: false;
        emailUnverified: boolean;
        email: string;
        mustChangePassword?: undefined;
        mfaSetupRequired?: undefined;
        mfaToken?: undefined;
    } | {
        success: true;
        mfa: false;
        mustChangePassword: boolean;
        emailUnverified?: undefined;
        email?: undefined;
        mfaSetupRequired?: undefined;
        mfaToken?: undefined;
    } | {
        success: true;
        mfa: false;
        mfaSetupRequired: boolean;
        emailUnverified?: undefined;
        email?: undefined;
        mustChangePassword?: undefined;
        mfaToken?: undefined;
    } | {
        success: true;
        mfa: false;
        emailUnverified?: undefined;
        email?: undefined;
        mustChangePassword?: undefined;
        mfaSetupRequired?: undefined;
        mfaToken?: undefined;
    } | {
        success: true;
        mfa: true;
        mfaToken: string;
        emailUnverified?: undefined;
        email?: undefined;
        mustChangePassword?: undefined;
        mfaSetupRequired?: undefined;
    } | {
        success: false;
        mfa: false;
        emailUnverified?: undefined;
        email?: undefined;
        mustChangePassword?: undefined;
        mfaSetupRequired?: undefined;
        mfaToken?: undefined;
    }>;
    verifyMfa: (mfaSessionToken: string, code: string) => Promise<{
        success: true;
    } | {
        success: false;
    }>;
    logout: () => Promise<void>;
};
//# sourceMappingURL=useAuth.d.ts.map