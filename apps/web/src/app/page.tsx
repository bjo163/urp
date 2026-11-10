import Link from "next/link";

const plannedApps = [
  { code: "CRM", name: "Customer operations", detail: "Contacts, pipeline and sales" },
  { code: "SCM", name: "Supply chain", detail: "Purchasing, inventory and logistics" },
  { code: "FIN", name: "Finance core", detail: "Double-entry accounting and controls" },
  { code: "OPS", name: "People & operations", detail: "HR, projects and manufacturing" }
];

export default function HomePage() {
  return (
    <main className="screen">
      <div className="grain" aria-hidden="true" />
      <header className="topbar">
        <Link className="wordmark" href="/" aria-label="URP home">
          <span className="wordmark-mark">U</span>
          <span>URP<span className="wordmark-period">.</span></span>
        </Link>
        <div className="topbar-meta">
          <span className="status-dot" />
          FOUNDATION BUILD
          <span className="version">0.1.0</span>
        </div>
      </header>

      <section className="hero">
        <p className="eyebrow"><span>01 / PLATFORM</span> BUSINESS OPERATING SYSTEM</p>
        <h1>
          Work gets
          <br />
          <span>re-engineered.</span>
        </h1>
        <div className="hero-bottom">
          <p className="hero-copy">
            One connected platform for the work behind a business.
            Built for clarity, control and serious operations.
          </p>
          <div className="hero-stamp" aria-label="Rust plus Next.js">
            <span>RUST</span>
            <b>×</b>
            <span>NEXT.JS</span>
          </div>
        </div>
      </section>

      <section className="platform-section" aria-labelledby="platform-heading">
        <div className="section-heading">
          <div>
            <p className="eyebrow">THE SYSTEM / 004 DOMAINS</p>
            <h2 id="platform-heading">One core. Many operations.</h2>
          </div>
          <span className="section-index">FOUNDATION / 00</span>
        </div>
        <div className="app-grid">
          {plannedApps.map((app, index) => (
            <article className="app-card" key={app.code}>
              <div className="app-card-top">
                <span className="app-code">{app.code}</span>
                <span className="app-number">0{index + 1}</span>
              </div>
              <h3>{app.name}</h3>
              <p>{app.detail}</p>
              <span className="app-state">PLANNED MODULE</span>
            </article>
          ))}
        </div>
      </section>

      <footer className="footer">
        <span>BUILT TO CONNECT THE WORK.</span>
        <span>URP / OPEN FOUNDATION</span>
      </footer>
    </main>
  );
}
