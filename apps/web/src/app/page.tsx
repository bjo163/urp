const foundationItems = [
  { index: '01', name: 'Kernel', detail: 'Typed domain and platform primitives', state: 'Building' },
  { index: '02', name: 'Model engine', detail: 'Metadata, relations and safe queries', state: 'Queued' },
  { index: '03', name: 'Module runtime', detail: 'Manifest, dependencies and lifecycle', state: 'Queued' },
  { index: '04', name: 'Business apps', detail: 'Connected operations, not isolated CRUD', state: 'Queued' },
];

export default function HomePage() {
  return (
    <main className="landing">
      <header className="topbar">
        <a className="wordmark" href="/" aria-label="URP home">
          <span className="wordmark-mark">U/</span>
          <span>URP</span>
        </a>
        <div className="topbar-meta">
          <span className="status-dot" />
          FOUNDATION MODE
          <span className="version-tag">v0.1.0-dev</span>
        </div>
      </header>

      <section className="hero">
        <div className="hero-kicker">
          <span>UNIVERSAL RADICAL PLATFORM</span>
          <span className="kicker-line" />
          <span>01 / THE OPERATING LAYER</span>
        </div>
        <h1>
          BUSINESS
          <br />
          WITHOUT
          <br />
          <span>THE BULLSH*T.</span>
        </h1>
        <div className="hero-bottom">
          <p>
            Satu platform untuk menggerakkan seluruh operasi bisnis.
            <br />
            Rust core. Modular by design. Radical by nature.
          </p>
          <div className="hero-stamp" aria-label="Built to be extended">
            <span>BUILT</span>
            <strong>TO</strong>
            <span>EXTEND</span>
          </div>
        </div>
      </section>

      <section className="foundation" aria-labelledby="foundation-title">
        <div className="section-heading">
          <div>
            <span className="eyebrow">SYSTEM / 000</span>
            <h2 id="foundation-title">THE FOUNDATION BOARD</h2>
          </div>
          <span className="board-count">04 TRACKS</span>
        </div>

        <div className="track-grid">
          {foundationItems.map((item) => (
            <article className="track-card" key={item.index}>
              <div className="track-card-top">
                <span className="track-index">{item.index}</span>
                <span className={item.state === 'Building' ? 'track-state active' : 'track-state'}>
                  {item.state}
                </span>
              </div>
              <h3>{item.name}</h3>
              <p>{item.detail}</p>
            </article>
          ))}
        </div>
      </section>

      <footer className="footer">
        <span>URP / BUSINESS OPERATING SYSTEM</span>
        <span>RUST × NEXT.JS / SELF-HOSTED FIRST</span>
      </footer>
    </main>
  );
}
