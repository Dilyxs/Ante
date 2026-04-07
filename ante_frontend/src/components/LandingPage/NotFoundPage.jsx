import { Link } from "react-router-dom";

const NotFoundPage = () => {
  return (
    <>
      <section className="landing-section not-found-section">
        <div className="landing-container not-found-card">
          <p className="not-found-kicker">404 / Not Found</p>
          <h1>The trail you followed has gone cold.</h1>
          <p>
            This path does not exist on the Monolith. Jump back to the primary route
            and continue browsing active protocol surfaces.
          </p>
          <Link className="table-action" to="/">
            Return to Main Page
          </Link>
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

export default NotFoundPage;
