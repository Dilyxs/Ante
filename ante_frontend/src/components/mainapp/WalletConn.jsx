import { useMemo } from "react";
import { clusterApiUrl } from "@solana/web3.js";
import {
  ConnectionProvider,
  WalletProvider,
  useWallet,
} from "@solana/wallet-adapter-react";
import {
  WalletModalProvider,
  useWalletModal,
} from "@solana/wallet-adapter-react-ui";
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import "@solana/wallet-adapter-react-ui/styles.css";

const WalletGate = ({ children }) => {
  const { connected, connecting } = useWallet();
  const { setVisible } = useWalletModal();

  const openWalletModal = () => setVisible(true);

  if (!connected) {
    return (
      <div className="relative min-h-screen overflow-hidden bg-[#f8f9fa] font-['Inter'] text-[#191c1d] antialiased">
        <nav className="fixed top-0 z-50 flex w-full items-center justify-between bg-slate-50/80 px-8 py-4 backdrop-blur-xl">
          <div className="font-['Manrope'] text-xl font-extrabold tracking-tight text-slate-950">
            Arctic Ledger
          </div>

          <div className="hidden items-center gap-8 md:flex">
            <a
              className="font-['Manrope'] text-sm font-semibold text-slate-500 transition-colors hover:text-slate-800"
              href="#"
            >
              Explorer
            </a>
            <a
              className="font-['Manrope'] text-sm font-semibold text-slate-500 transition-colors hover:text-slate-800"
              href="#"
            >
              Governance
            </a>
            <a
              className="font-['Manrope'] text-sm font-semibold text-slate-500 transition-colors hover:text-slate-800"
              href="#"
            >
              Bridge
            </a>
          </div>

          <button
            className="rounded-lg bg-[#030813] px-6 py-2 font-['Manrope'] text-sm font-bold text-white transition-all hover:bg-[#1a202c]"
            onClick={openWalletModal}
          >
            {connecting ? "Connecting..." : "Connect Wallet"}
          </button>
        </nav>

        <div className="flex min-h-screen flex-col">
          <main className="flex flex-1 items-center justify-center bg-[linear-gradient(135deg,#f8f9fa_0%,#edeeef_100%)] px-6 pt-20">
            <div className="relative w-full max-w-lg">
              <div className="absolute -left-24 -top-24 h-64 w-64 rounded-full bg-[#d2e4ff]/50 blur-[100px]" />
              <div className="absolute -bottom-24 -right-24 h-64 w-64 rounded-full bg-[#dde2f3]/40 blur-[100px]" />

              <div className="relative rounded-xl border border-[#c6c6cc]/20 bg-white p-10 shadow-[0_40px_80px_-15px_rgba(0,0,0,0.04)] md:p-14">
                <div className="mb-12 text-center">
                  <div className="mb-6 inline-flex h-16 w-16 items-center justify-center rounded-full bg-[#f3f4f5]">
                    <span className="material-symbols-outlined text-3xl text-[#030813]">
                      lock
                    </span>
                  </div>
                  <h1 className="mb-3 font-['Manrope'] text-3xl font-extrabold tracking-tight text-[#030813] md:text-4xl">
                    Secure Access
                  </h1>
                  <p className="mx-auto max-w-xs leading-relaxed text-[#45474c]">
                    Connect your Solana wallet to manage your assets on the
                    Arctic Ledger.
                  </p>
                </div>

                <div className="space-y-4">
                  <button
                    className="group flex w-full items-center justify-between rounded-lg bg-[#030813] p-5 text-white transition-all duration-300 hover:bg-[#1a202c]"
                    onClick={openWalletModal}
                  >
                    <div className="flex items-center gap-4">
                      <span className="material-symbols-outlined">
                        account_balance_wallet
                      </span>
                      <span className="font-['Manrope'] text-lg font-bold tracking-tight">
                        Connect Solana Wallet
                      </span>
                    </div>
                    <span className="material-symbols-outlined transition-transform group-hover:translate-x-1">
                      arrow_forward
                    </span>
                  </button>

                  <div className="flex items-center justify-center gap-3 py-4">
                    {connecting ? (
                      <>
                        <div className="h-4 w-4 animate-spin rounded-full border-2 border-[#c1c6d7] border-t-[#0061a5]" />
                        <span className="text-xs font-semibold uppercase tracking-wide text-[#7d8a90]">
                          Checking connection...
                        </span>
                      </>
                    ) : (
                      <>
                        <span className="h-2 w-2 rounded-full bg-[#0061a5]" />
                        <span className="text-xs font-semibold uppercase tracking-wide text-[#7d8a90]">
                          Awaiting wallet connection
                        </span>
                      </>
                    )}
                  </div>
                </div>

                <div className="mt-12 border-t border-[#edeeef] pt-8">
                  <p className="mb-6 text-center text-xs font-semibold uppercase tracking-[0.2em] text-[#45474c]">
                    Supported Ecosystems
                  </p>
                  <div className="grid grid-cols-3 gap-4">
                    <div className="group cursor-pointer rounded-lg p-4 transition-colors hover:bg-[#f3f4f5]">
                      <div className="flex flex-col items-center gap-2">
                        <span className="material-symbols-outlined text-[#7d8a90] transition-colors group-hover:text-[#030813]">
                          token
                        </span>
                        <span className="text-[10px] font-bold uppercase tracking-tight">
                          Phantom
                        </span>
                      </div>
                    </div>
                    <div className="group cursor-pointer rounded-lg p-4 transition-colors hover:bg-[#f3f4f5]">
                      <div className="flex flex-col items-center gap-2">
                        <span className="material-symbols-outlined text-[#7d8a90] transition-colors group-hover:text-[#030813]">
                          account_balance
                        </span>
                        <span className="text-[10px] font-bold uppercase tracking-tight">
                          Solflare
                        </span>
                      </div>
                    </div>
                    <div className="group cursor-pointer rounded-lg p-4 transition-colors hover:bg-[#f3f4f5]">
                      <div className="flex flex-col items-center gap-2">
                        <span className="material-symbols-outlined text-[#7d8a90] transition-colors group-hover:text-[#030813]">
                          database
                        </span>
                        <span className="text-[10px] font-bold uppercase tracking-tight">
                          Ledger
                        </span>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="mt-10 flex items-start gap-3 rounded-lg bg-[#f3f4f5] p-4">
                  <span className="material-symbols-outlined text-lg text-[#0061a5]">
                    verified_user
                  </span>
                  <p className="text-[11px] leading-normal text-[#45474c]">
                    By connecting, you agree to the Arctic Ledger Protocol
                    terms. Your private keys never leave your device.
                    Connection is encrypted via AES-256.
                  </p>
                </div>
              </div>

              <div className="mt-8 text-center">
                <div className="inline-flex items-center gap-2 opacity-40">
                  <div className="h-1.5 w-1.5 animate-pulse rounded-full bg-[#0061a5]" />
                  <span className="text-[10px] font-bold uppercase tracking-[0.2em] text-[#030813]">
                    Arctic Network Status: Operational
                  </span>
                </div>
              </div>
            </div>
          </main>

          <footer className="w-full bg-slate-100 px-8 py-12">
            <div className="mx-auto flex max-w-7xl flex-col items-center justify-between md:flex-row">
              <div className="mb-4 text-xs tracking-wide text-slate-400 md:mb-0">
                © 2024 Arctic Monolith. Secured by Ethereum.
              </div>
              <div className="flex gap-6">
                <a
                  className="text-xs tracking-wide text-slate-400 opacity-80 transition-opacity hover:text-slate-600 hover:opacity-100"
                  href="#"
                >
                  Privacy Policy
                </a>
                <a
                  className="text-xs tracking-wide text-slate-400 opacity-80 transition-opacity hover:text-slate-600 hover:opacity-100"
                  href="#"
                >
                  Terms of Service
                </a>
                <a
                  className="text-xs tracking-wide text-slate-400 opacity-80 transition-opacity hover:text-slate-600 hover:opacity-100"
                  href="#"
                >
                  Security Audit
                </a>
              </div>
            </div>
          </footer>
        </div>

        <div className="pointer-events-none fixed inset-0 -z-10 opacity-5">
          <img
            alt=""
            className="h-full w-full object-cover"
            src="https://lh3.googleusercontent.com/aida-public/AB6AXuAnJQR14cxu8wgXu9Gd5Yx7GOcqmVyNfyNuawhyantL7fvpMVyMO5cwGQsPpqiR7znikFETUvdmR-O62cSJMfjzZng_DgmuE50arvV45eKMyt224rFQTKJMVEpNiAo_8Bmn9vC5QTPybccYn7H9CZ8EkR2-FucIWAWXjOb9X1dfY8LFgs7FymESaJ-u-iKlJ7KOb63UIFdXCzq0p-iN4Jsnf4kHxLCb2swQxQHMQ2pPTrGYfbogney7eUIbcu7QQYI-ar5w7TXMxygb"
          />
        </div>
      </div>
    );
  }

  return children;
};

const WalletConn = ({ children }) => {
  const endpoint = useMemo(
    () => import.meta.env.VITE_SOLANA_RPC_URL || clusterApiUrl("devnet"),
    [],
  );

  const wallets = useMemo(
    () => [new PhantomWalletAdapter(), new SolflareWalletAdapter()],
    [],
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <WalletGate>{children}</WalletGate>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
};

export default WalletConn;
