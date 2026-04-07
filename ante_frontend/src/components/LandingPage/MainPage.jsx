import { Link, useNavigate } from "react-router-dom";

const featureCards = [
  {
    title: "OpenEnded",
    description:
      "Propose theoretical vulnerabilities or research questions. Incentivize the brightest minds to explore the unknown boundaries of modern cryptography.",
    image:
      "https://images.unsplash.com/photo-1545987796-200677ee1011?auto=format&fit=crop&w=1400&q=80",
  },
  {
    title: "DirectAnswer",
    description:
      "Precise challenges with deterministic solutions. From breaking hash functions to finding private keys, rewards are unlocked instantly upon proof submission.",
    image:
      "https://images.unsplash.com/photo-1510511459019-5dda7724fd87?auto=format&fit=crop&w=1400&q=80",
  },
];

const domains = [
  "Number Theory",
  "CryptoPuzzles",
  "Reverse Engineering",
  "Zero Knowledge",
];
//TODO: fetch from backend if len < 3 use this.
const challengeRows = [
  {
    name: "Primes of Order 7",
    hash: "HASH: 0x4e2...f9a",
    category: "Number Theory",
    reward: "12.5 ETH",
  },
  {
    name: "ZK-Rollup Efficiency Proof",
    hash: "HASH: 0x1a8...3d2",
    category: "Zero Knowledge",
    reward: "45,000 USDC",
  },
  {
    name: "Elliptic Curve Collision",
    hash: "HASH: 0xc91...e0b",
    category: "CryptoPuzzles",
    reward: "8.0 ETH",
  },
];

const domainIcons = ["✣", "✦", "⌁", "⌖"];

const MainPage = () => {
  const navigate = useNavigate();
  return (
    <>
      <section className="landing-section hero-section">
        <div className="landing-container landing-hero-inner">
          <div className="landing-pill">Decentralized Proofs</div>
          <h1 className="landing-title">
            The Future of
            <br />
            Cryptographic <span>Bounties.</span>
          </h1>
          <p className="landing-subtitle">
            A permissionless protocol for posting and solving hard mathematical
            and cryptographic challenges. Secure, transparent, and immutable.
          </p>
          <div className="landing-actions">
            <button
              onClick={() => {
                navigate("/dashboard");
              }}
              className="action-primary"
              type="button"
            >
              Launch Dashboard
            </button>
          </div>
        </div>
      </section>

      <section className="landing-section feature-section">
        <div className="landing-container feature-grid">
          {featureCards.map((card, index) => (
            <article className="feature-card" key={card.title}>
              <div
                className={
                  index === 0 ? "feature-icon is-light" : "feature-icon"
                }
              >
                ◉
              </div>
              <h2>{card.title}</h2>
              <p>{card.description}</p>
              <img alt={card.title} src={card.image} />
            </article>
          ))}
        </div>
      </section>

      <section className="landing-section domain-section">
        <div className="landing-container domain-wrap">
          <h2>Explore Domains</h2>
          <p>
            The frontier of cryptographic innovation, organized by specialty.
          </p>
          <div className="domain-grid">
            {domains.map((domain, index) => (
              <article className="domain-card" key={domain}>
                <span className="domain-icon">{domainIcons[index]}</span>
                <h3>{domain}</h3>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="landing-section process-section">
        <div className="landing-container process-grid">
          <article className="process-card">
            <span>01</span>
            <h3>Post</h3>
            <p>
              Define your challenge parameters, set the bounty amount in ETH or
              USDC, and lock rewards into our audited smart contracts.
            </p>
          </article>
          <article className="process-card">
            <span>02</span>
            <h3>Solve</h3>
            <p>
              Analysts and cryptographers globally compete to find the solution.
              Solutions are submitted as cryptographic proofs.
            </p>
          </article>
          <article className="process-card">
            <span>03</span>
            <h3>Reveal</h3>
            <p>
              Once verified, the bounty is automatically released to the solver.
              The solution is permanently recorded on the Monolith.
            </p>
          </article>
        </div>
      </section>

      <section className="landing-section table-section">
        <div className="landing-container">
          <div className="table-title-row">
            <div>
              <h2>Active Challenges</h2>
              <p>Live bounties waiting for a solution.</p>
            </div>
            <Link className="table-link" to="/howitworks">
              View Protocol Flow
            </Link>
          </div>

          <div className="landing-table-canvas">
            <table className="landing-table">
              <thead>
                <tr>
                  <th>Challenge Name</th>
                  <th>Category</th>
                  <th>Reward</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {challengeRows.map((row) => (
                  <tr key={row.name}>
                    <td>
                      <strong>{row.name}</strong>
                      <small>{row.hash}</small>
                    </td>
                    <td>
                      <span className="category-pill">{row.category}</span>
                    </td>
                    <td className="reward-cell">{row.reward}</td>
                    <td>
                      <button className="table-action" type="button">
                        Solve
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <section className="landing-section cta-section">
        <div className="landing-container">
          <div className="cta-card">
            <h2>Ready to secure the future?</h2>
            <p>
              Whether you are a protocol looking for formal verification or a
              researcher seeking challenges, the Monolith awaits.
            </p>
            <Link className="cta-button" to="/dashboard">
              Get Started Now
            </Link>
          </div>
        </div>
      </section>

      <footer className="site-footer">
        <div className="landing-container footer-inner">
          <p>© 2024 Ante. Built on the Monolith.</p>
          <div className="footer-links">
            <a href="#">Docs</a>
            <a href="#">Discord</a>
            <a href="#">Status</a>
          </div>
        </div>
      </footer>
    </>
  );
};

export default MainPage;
