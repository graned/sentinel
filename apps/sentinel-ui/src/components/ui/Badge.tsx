import styles from "./Badge.module.css";

export type BadgeVariant = "active" | "inactive" | "danger" | "warning" | "blue" | "muted";

interface BadgeProps {
  variant?: BadgeVariant;
  children: React.ReactNode;
}

export function Badge({ variant = "muted", children }: BadgeProps) {
  return <span className={`${styles.badge} ${styles[variant]}`}>{children}</span>;
}
