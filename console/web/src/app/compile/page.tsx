
import CompileForm from "../../components/CompileForm";

export default function CompilePage() {
  return (
    <div className="panel">
      <div className="h1">Compile</div>
      <div className="small">Submit a structure payload and get schema/manifest/proof.</div>
      <div style={{ height: 12 }} />
      <CompileForm />
    </div>
  );
}
