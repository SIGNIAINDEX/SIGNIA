
import Link from "next/link";

export default function Page() {
  return (
    <div className="row">
      <div className="col">
        <div className="panel">
          <h1 className="h1">SIGNIA Console</h1>
          <p className="p">
            A minimal operator UI for compiling structures into deterministic schemas, producing proofs,
            and interfacing with the Solana registry.
          </p>

          <div className="row">
            <Link className="btn primary" href="/compile">Compile</Link>
            <Link className="btn" href="/verify">Verify</Link>
            <Link className="btn" href="/registry">Registry</Link>
            <Link className="btn" href="/interface">Interface</Link>
          </div>
        </div>

        <div className="panel" style={{ marginTop: 12 }}>
          <div className="h2">Expected Services</div>
          <div className="kv">
            <div className="small">signia-api</div>
            <div><code>NEXT_PUBLIC_SIGNIA_API</code> (default <code>http://localhost:8080</code>)</div>
            <div className="small">interface</div>
            <div><code>NEXT_PUBLIC_SIGNIA_INTERFACE</code> (default <code>http://localhost:7070</code>)</div>
          </div>
        </div>
      </div>

      <div className="col">
        <div className="panel">
          <div className="h2">Operational Notes</div>
          <ul className="p" style={{ margin: 0, paddingLeft: 18 }}>
            <li>Inputs are canonicalized server-side to produce stable hashes.</li>
            <li>Artifacts are content-addressed and retrievable by id.</li>
            <li>Registry actions are surfaced through the API status endpoint.</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
