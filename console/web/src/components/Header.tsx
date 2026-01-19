
import Link from "next/link";

export default function Header() {
  return (
    <div className="panel" style={{ padding: 14, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
      <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
        <div style={{ width: 10, height: 10, borderRadius: 999, background: "var(--accent)" }} />
        <div>
          <div style={{ fontWeight: 700, letterSpacing: 0.4 }}>SIGNIA Console</div>
          <div className="small">Structure → verifiable on-chain forms</div>
        </div>
      </div>

      <div style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
        <Link className="btn" href="/">Home</Link>
        <Link className="btn" href="/compile">Compile</Link>
        <Link className="btn" href="/verify">Verify</Link>
        <Link className="btn" href="/registry">Registry</Link>
        <Link className="btn primary" href="/interface">Interface</Link>
      </div>
    </div>
  );
}
