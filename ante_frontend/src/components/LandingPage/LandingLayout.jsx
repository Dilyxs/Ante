import {
  Link,
  NavLink,
  Outlet,
  useLocation,
  useNavigate,
} from "react-router-dom";
import "./landing.css";

const navItems = [
  { to: "/", label: "Explore", end: true },
  { to: "/howitworks", label: "How It Works" },
];

const LandingLayout = () => {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const isHowItWorks = pathname === "/howitworks";
  const showAvatar = pathname === "/";

  return (
    <div className="page-shell">
      <nav className="landing-nav sticky top-0 z-50 bg-slate-50/80 backdrop-blur-xl">
        <div className="landing-container landing-nav-inner">
          <Link className="brand" to="/">
            Ante
          </Link>

          <div className="nav-links" role="navigation" aria-label="Main">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.end}
                className={({ isActive }) =>
                  isActive ? "nav-link nav-link-active" : "nav-link"
                }
              >
                {item.label}
              </NavLink>
            ))}
          </div>

          <div className="nav-right">
            <button
              onClick={() => {
                navigate("/dashboard");
              }}
              className="wallet-button"
              type="button"
            >
              Connect Wallet
            </button>
            {showAvatar ? <span className="avatar-dot" aria-hidden="true" /> : null}
          </div>
        </div>
      </nav>

      <main className={isHowItWorks ? "page-main how-main" : "page-main"}>
        <Outlet />
      </main>
    </div>
  );
};

export default LandingLayout;
