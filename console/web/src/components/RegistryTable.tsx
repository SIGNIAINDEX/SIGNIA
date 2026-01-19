
export default function RegistryTable({ data }: { data: any }) {
  const rows: Array<{ k: string; v: any }> = [
    { k: "programId", v: data?.programId },
    { k: "cluster", v: data?.cluster },
    { k: "registryPda", v: data?.registryPda },
    { k: "enabled", v: data?.enabled },
    { k: "notes", v: data?.notes },
  ];

  return (
    <div className="panel">
      <div className="h2">Registry</div>
      <div className="kv">
        {rows.map((r) => (
          <div key={r.k} style={{ display: "contents" }}>
            <div className="small">{r.k}</div>
            <div style={{ wordBreak: "break-word" }}>
              {typeof r.v === "string" ? <code>{r.v}</code> : JSON.stringify(r.v)}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
