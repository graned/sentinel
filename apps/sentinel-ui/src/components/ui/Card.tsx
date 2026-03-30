import styles from "./Card.module.css";

interface CardProps {
  title?: string;
  children: React.ReactNode;
  className?: string;
}

export function Card({ title, children, className }: CardProps) {
  return (
    <div className={`${styles.card} ${className ?? ""}`}>
      {title && <h2 className={styles.cardTitle}>{title}</h2>}
      <div className={styles.cardBody}>{children}</div>
    </div>
  );
}
