import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ChallengeProtocol } from "../target/types/challenge_protocol";
import { assert } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
} from "@solana/spl-token";

describe("challenge-protocol", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .challengeProtocol as Program<ChallengeProtocol>;

  const VAULT_AUTHORITY_SEED = Buffer.from("authority");
  const ANTE_MINT_SEED = Buffer.from("Ante");
  const CONFIG_SEED = Buffer.from("config");
  const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
  const ASSOCIATED_TOKEN_PROGRAM = ASSOCIATED_TOKEN_PROGRAM_ID;
  const VAULT_MINT_DECIMALS = 6;

  it("initialize vault", async () => {
    const provider = anchor.getProvider();
    const wallet = provider.wallet.publicKey;

    const [vaultAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [VAULT_AUTHORITY_SEED],
      program.programId
    );
    const [anteMintPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [ANTE_MINT_SEED],
      program.programId
    );
    const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [CONFIG_SEED],
      program.programId
    );

    const vaultAta = getAssociatedTokenAddressSync(
      anteMintPda,
      vaultAuthorityPda,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );

    await program.methods
      .initialize()
      .accounts({
        signer: wallet,
        vaultAuthority: vaultAuthorityPda,
        mint: anteMintPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const configAccount = await program.account.config.fetch(configPda);
    assert.ok(configAccount.admin.equals(wallet));

    const mintInfo = await getMint(provider.connection, anteMintPda);
    assert.equal(mintInfo.decimals, VAULT_MINT_DECIMALS);
    assert.ok(mintInfo.mintAuthority?.equals(vaultAuthorityPda));

    const vaultAtaAccount = await getAccount(provider.connection, vaultAta);
    assert.equal(vaultAtaAccount.amount, BigInt(0));
    assert.ok(vaultAtaAccount.owner.equals(vaultAuthorityPda));
    assert.ok(vaultAtaAccount.mint.equals(anteMintPda));
  });
});
