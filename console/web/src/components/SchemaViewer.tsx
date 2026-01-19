
export default function SchemaViewer({ schema }: { schema: unknown }) {
  return (
    <div className="panel">
      <div className="h2">Schema</div>
      <pre style={{ margin: 0, overflowX: "auto" }}>{JSON.stringify(schema, null, 2)}</pre>
    </div>
  );
}
