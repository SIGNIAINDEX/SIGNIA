
export type ArtifactRow = { id: string; size?: number; kind?: string };

export default function ArtifactList({ items, onPick }: { items: ArtifactRow[]; onPick?: (id: string) => void }) {
  if (!items?.length) {
    return (
      <div className="panel">
        <div className="h2">Artifacts</div>
        <div className="small">No artifacts returned.</div>
      </div>
    );
  }

  return (
    <div className="panel">
      <div className="h2">Artifacts</div>
      <table className="table">
        <thead>
          <tr>
            <th>id</th>
            <th>kind</th>
            <th>size</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {items.map((a) => (
            <tr key={a.id}>
              <td><code>{a.id}</code></td>
              <td>{a.kind || "-"}</td>
              <td>{typeof a.size === "number" ? a.size : "-"}</td>
              <td>
                {onPick ? (
                  <button className="btn" onClick={() => onPick(a.id)}>Open</button>
                ) : (
                  <span className="small">—</span>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
