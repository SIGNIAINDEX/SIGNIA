
export default function ProofViewer({ proof }: { proof: unknown }) {
  return (
    <div className="panel">
      <div className="h2">Proof</div>
      <pre style={{ margin: 0, overflowX: "auto" }}>{JSON.stringify(proof, null, 2)}</pre>
    </div>
  );
}
