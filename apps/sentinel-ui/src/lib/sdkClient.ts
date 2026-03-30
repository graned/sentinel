import { SentinelAuthClient } from "@sentinel/auth-sdk";

export const sentinelClient = new SentinelAuthClient({
  baseUrl: import.meta.env.VITE_API_URL ?? "http://localhost:8080",
});
