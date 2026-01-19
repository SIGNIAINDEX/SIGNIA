
import "../styles/globals.css";
import Header from "../components/Header";
import Footer from "../components/Footer";

export const metadata = {
  title: "SIGNIA Console",
  description: "Compile real-world structures into verifiable forms.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <div className="container" style={{ display: "flex", flexDirection: "column", gap: 14 }}>
          <Header />
          {children}
          <Footer />
        </div>
      </body>
    </html>
  );
}
