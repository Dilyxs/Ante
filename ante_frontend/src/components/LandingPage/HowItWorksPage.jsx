import { useState, useEffect } from "react";

const steps = [
  {
    id: "01",
    title: "Post",
    body: "Define challenge parameters, escrow reward funds, and publish immutable rules to the protocol.",
  },
  {
    id: "02",
    title: "Solve",
    body: "Researchers compete globally by submitting cryptographic proofs against the posted constraints.",
  },
  {
    id: "03",
    title: "Reveal",
    body: "Verified solutions trigger automatic reward distribution and permanent public ledger recording.",
  },
];

const rails = [
  "Challenge specification hash",
  "Escrow vault proof",
  "Submission signature",
  "Verification receipt",
  "Reward distribution event",
];

const HowItWorksPage = () => {
  const [waitlistClicked, setWaitlistClicked] = useState(false);

  useEffect(() => {
    if (!waitlistClicked) return;
    const onKey = (e) => {
      if (e.key === "Escape") setWaitlistClicked(false);
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [waitlistClicked]);

  return (
    <>
      <section className="how-hero-section">
        <div className="how-bg-ice" aria-hidden="true" />
        <div className="how-container">
          <p className="how-brand">Ante</p>
          <h1 className="how-title">
            The future of
            <br />
            <span>cryptographic bounties</span> is
            <br />
            arriving.
          </h1>
          <p className="how-subtitle">
            We are carving a new paradigm in trustless incentives. High-stakes
            precision, Nordic clarity, and total sovereign security.
          </p>

          <form
            className="how-waitlist"
            onSubmit={(event) => event.preventDefault()}
          >
            <input placeholder="Enter your wallet or email" type="email" />
            <button
              type="submit"
              className="group inline-flex cursor-pointer items-center justify-center rounded-xl bg-gradient-to-r from-slate-950 to-slate-800 px-5 py-3 text-sm font-extrabold text-white shadow-[0_14px_35px_rgba(3,8,19,0.22)] transition-all duration-200 hover:from-slate-900 hover:to-slate-700 hover:shadow-[0_18px_45px_rgba(3,8,19,0.28)] active:scale-95"
              onClick={() => setWaitlistClicked(true)}
            >
              <span className="mr-2 inline-block h-2 w-2 rounded-full bg-sky-300 transition-colors duration-200 group-hover:bg-sky-200" />
              Join Waitlist
            </button>
          </form>

          <p className="how-lock">By Team Ante</p>
        </div>
      </section>

      {waitlistClicked ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center px-4">
          <div
            className="absolute inset-0 bg-black/40 backdrop-blur-sm"
            onClick={() => setWaitlistClicked(false)}
          />

          <div
            role="status"
            aria-live="polite"
            className="relative z-10 w-[min(680px,92vw)] rounded-2xl bg-gradient-to-br from-white/6 to-white/3 p-6 backdrop-blur-md shadow-2xl border border-white/6 transform transition-all duration-300"
          >
            <div className="flex w-full flex-col items-start gap-4 text-left md:flex-row md:items-center md:justify-between">
              <div className="flex flex-col gap-1">
                <h3 className="text-lg font-manrope font-extrabold text-white">
                  You're on the waitlist
                </h3>

                <p className="max-w-[56ch] text-sm text-slate-200">
                  Thanks — we saved your spot. You'll receive an email or a
                  wallet notification when the next challenge window opens.
                </p>
              </div>

              <div className="flex shrink-0 items-center gap-3">
                <button
                  onClick={
                    //TODO: make the button prettier
                    () => setWaitlistClicked(false)
                  }
                  className="rounded-lg bg-white/10 px-4 py-2 text-sm font-semibold text-white hover:bg-white/20"
                >
                  Done
                </button>
              </div>
            </div>
          </div>
        </div>
      ) : null}

      <section className="landing-section section-soft">
        <div className="landing-container steps-grid">
          {steps.map((step) => (
            <article className="step-card" key={step.id}>
              <span>{step.id}</span>
              <h2>{step.title}</h2>
              <p>{step.body}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="landing-section">
        <div className="landing-container rails-card">
          <h2>Verification Rails</h2>
          <p>
            These protocol artifacts create the trustless paper trail from
            challenge publication through payout.
          </p>
          <ul className="rails-list">
            {rails.map((rail) => (
              <li key={rail}>{rail}</li>
            ))}
          </ul>
        </div>
      </section>

      <footer className="how-footer">
        <div className="how-footer-inner">
          <div className="how-footer-brand">Ante</div>
          <div className="how-footer-links">
            <a href="#">Twitter</a>
            <a href="#">GitHub</a>
            <a href="#">Docs</a>
          </div>
          <p>© 2026 Team Ante</p>
        </div>
      </footer>
    </>
  );
};

export default HowItWorksPage;
