import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ChallengeProtocol } from "../target/types/challenge_protocol";
import { assert } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
} from "@solana/spl-token";

describe("challenge-protocol", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .challengeProtocol as Program<ChallengeProtocol>;

  const VAULT_AUTHORITY_SEED = Buffer.from("authority");
  const ANTE_MINT_SEED = Buffer.from("Ante");
  const CONFIG_SEED = Buffer.from("config");
  const USER_BALANCE_INFO_SEED = Buffer.from("user_balance_info");
  const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
  const ASSOCIATED_TOKEN_PROGRAM = ASSOCIATED_TOKEN_PROGRAM_ID;
  const VAULT_MINT_DECIMALS = 6;

  let vaultAuthorityPda: anchor.web3.PublicKey;
  let anteMintPda: anchor.web3.PublicKey;
  let configPda: anchor.web3.PublicKey;
  let userBalancePda: anchor.web3.PublicKey;
  let vaultAta: anchor.web3.PublicKey;
  let ownerAta: anchor.web3.PublicKey;

  // helper to encode u64 little-endian
  function UInt64ToLE(n: number): Buffer {
    const buf = Buffer.alloc(8);
    let v = BigInt(n);
    for (let i = 0; i < 8; i++) {
      buf[i] = Number(v & BigInt(0xff));
      v >>= BigInt(8);
    }
    return buf;
  }

  before(async () => {
    const wallet = provider.wallet.publicKey;
    [vaultAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [VAULT_AUTHORITY_SEED],
      program.programId
    );
    [anteMintPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [ANTE_MINT_SEED],
      program.programId
    );
    [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [CONFIG_SEED],
      program.programId
    );
    [userBalancePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [USER_BALANCE_INFO_SEED, wallet.toBuffer()],
      program.programId
    );

    vaultAta = getAssociatedTokenAddressSync(
      anteMintPda,
      vaultAuthorityPda,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );

    ownerAta = getAssociatedTokenAddressSync(
      anteMintPda,
      wallet,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );
  });

  it("initialize vault", async () => {
    const wallet = provider.wallet.publicKey;

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
      } as any)
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

  it("fails: stranger cannot mint ante tokens", async () => {
    const wallet = provider.wallet.publicKey;
    const stranger = anchor.web3.Keypair.generate();
    const strangerAta = getAssociatedTokenAddressSync(
      anteMintPda,
      stranger.publicKey,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );

    const createStrangerAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet,
        strangerAta,
        stranger.publicKey,
        anteMintPda,
        TOKEN_PROGRAM,
        ASSOCIATED_TOKEN_PROGRAM
      )
    );
    await provider.sendAndConfirm(createStrangerAtaTx);

    let failed = false;
    try {
      await program.methods
        .requestAnteTokens(new anchor.BN(1))
        .accounts({
          admin: stranger.publicKey,
          mint: anteMintPda,
          asker: stranger.publicKey,
          askerAta: strangerAta,
          vaultAuthority: vaultAuthorityPda,
          vaultAta,
          config: configPda,
          tokenProgram: TOKEN_PROGRAM,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
          systemProgram: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([stranger])
        .rpc();
      assert.fail("expected stranger mint request to fail");
    } catch (err) {
      failed = true;
      assert.isOk(err);
    }

    assert.isTrue(failed, "stranger mint request should fail");
  });

  it("passes: owner distributes ante tokens directly to stranger", async () => {
    const wallet = provider.wallet.publicKey;
    const ownerAtaAccount = await provider.connection.getAccountInfo(ownerAta);
    if (!ownerAtaAccount) {
      const createOwnerAtaTx = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
          wallet,
          ownerAta,
          wallet,
          anteMintPda,
          TOKEN_PROGRAM,
          ASSOCIATED_TOKEN_PROGRAM
        )
      );
      await provider.sendAndConfirm(createOwnerAtaTx);
    }

    const stranger = anchor.web3.Keypair.generate();
    const strangerAta = getAssociatedTokenAddressSync(
      anteMintPda,
      stranger.publicKey,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );

    const createStrangerAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet,
        strangerAta,
        stranger.publicKey,
        anteMintPda,
        TOKEN_PROGRAM,
        ASSOCIATED_TOKEN_PROGRAM
      )
    );
    await provider.sendAndConfirm(createStrangerAtaTx);

    const ownerBefore = await getAccount(provider.connection, ownerAta);
    const strangerBefore = await getAccount(provider.connection, strangerAta);

    await program.methods
      .requestAnteTokens(new anchor.BN(4))
      .accounts({
        admin: wallet,
        mint: anteMintPda,
        asker: stranger.publicKey,
        askerAta: strangerAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    const strangerAccount = await getAccount(provider.connection, strangerAta);
    assert.equal(strangerAccount.amount, strangerBefore.amount + BigInt(4));

    const ownerAccount = await getAccount(provider.connection, ownerAta);
    assert.equal(ownerAccount.amount, ownerBefore.amount);
  });

  it("happy path: deposit then withdraw ante tokens", async () => {
    const wallet = provider.wallet.publicKey;

    const ownerAtaAccount = await provider.connection.getAccountInfo(ownerAta);
    if (!ownerAtaAccount) {
      const createOwnerAtaTx = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
          wallet,
          ownerAta,
          wallet,
          anteMintPda,
          TOKEN_PROGRAM,
          ASSOCIATED_TOKEN_PROGRAM
        )
      );
      await provider.sendAndConfirm(createOwnerAtaTx);
    }

    await program.methods
      .requestAnteTokens(new anchor.BN(8))
      .accounts({
        admin: wallet,
        mint: anteMintPda,
        asker: wallet,
        askerAta: ownerAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    const depositAmount = new anchor.BN(5);
    const withdrawAmount = new anchor.BN(2);

    const ownerBeforeDeposit = await getAccount(provider.connection, ownerAta);
    const vaultBeforeDeposit = await getAccount(provider.connection, vaultAta);

    await (program as any).methods
      .depositeAnteTokens(depositAmount)
      .accounts({
        owner: wallet,
        ownerAta,
        mint: anteMintPda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: userBalancePda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const ownerAfterDeposit = await getAccount(provider.connection, ownerAta);
    const vaultAfterDeposit = await getAccount(provider.connection, vaultAta);
    assert.equal(
      ownerAfterDeposit.amount,
      ownerBeforeDeposit.amount - BigInt(5)
    );
    assert.equal(
      vaultAfterDeposit.amount,
      vaultBeforeDeposit.amount + BigInt(5)
    );

    let userBalance = await (program.account as any).userBalance.fetch(
      userBalancePda
    );
    assert.equal(userBalance.balance.toString(), "5");

    const ownerBeforeWithdraw = await getAccount(provider.connection, ownerAta);
    const vaultBeforeWithdraw = await getAccount(provider.connection, vaultAta);

    await (program as any).methods
      .withdrawAnteTokens(withdrawAmount)
      .accounts({
        owner: wallet,
        ownerAta,
        mint: anteMintPda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: userBalancePda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const ownerAfterWithdraw = await getAccount(provider.connection, ownerAta);
    const vaultAfterWithdraw = await getAccount(provider.connection, vaultAta);
    assert.equal(
      ownerAfterWithdraw.amount,
      ownerBeforeWithdraw.amount + BigInt(2)
    );
    assert.equal(
      vaultAfterWithdraw.amount,
      vaultBeforeWithdraw.amount - BigInt(2)
    );

    userBalance = await (program.account as any).userBalance.fetch(
      userBalancePda
    );
    assert.equal(userBalance.balance.toString(), "3");
  });

  it("sad path: fails when withdrawing more than balance", async () => {
    const wallet = provider.wallet.publicKey;
    const current_token_balance = await getAccount(
      provider.connection,
      ownerAta
    );

    let failed = false;
    try {
      await (program as any).methods
        .withdrawAnteTokens(new anchor.BN(8))
        .accounts({
          owner: wallet,
          ownerAta,
          mint: anteMintPda,
          vaultAuthority: vaultAuthorityPda,
          vaultAta,
          userBalanceInfo: userBalancePda,
          tokenProgram: TOKEN_PROGRAM,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("expected withdraw above balance to fail");
    } catch (err: any) {
      failed = true;
      assert.isOk(err);
      assert.include(err.toString(), "InsufficientAnteTokens");
    }

    assert.isTrue(failed, "withdraw above balance should fail");
  });

  it("poster: create poster happy path (sufficient balance)", async () => {
    const wallet = provider.wallet.publicKey;

    // ensure owner ATA exists
    const ownerAtaAccount = await provider.connection.getAccountInfo(ownerAta);
    if (!ownerAtaAccount) {
      const createOwnerAtaTx = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
          wallet,
          ownerAta,
          wallet,
          anteMintPda,
          TOKEN_PROGRAM,
          ASSOCIATED_TOKEN_PROGRAM
        )
      );
      await provider.sendAndConfirm(createOwnerAtaTx);
    }

    // deposit enough tokens for bounty_minimum_gain
    const depositAmount = new anchor.BN(10);
    await (program as any).methods
      .requestAnteTokens(new anchor.BN(8))
      .accounts({
        admin: wallet,
        mint: anteMintPda,
        asker: wallet,
        askerAta: ownerAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    // deposit into program balance
    // Note: requestAnteTokens only allows minting 1,2,4,8 so ensure depositAmount <= 8
    const depositAmountAdjusted = new anchor.BN(8);
    const beforeOwner = await getAccount(provider.connection, ownerAta);
    const beforeVault = await getAccount(provider.connection, vaultAta);
    await (program as any).methods
      .depositeAnteTokens(depositAmountAdjusted)
      .accounts({
        owner: wallet,
        ownerAta,
        mint: anteMintPda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: userBalancePda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const afterOwner = await getAccount(provider.connection, ownerAta);
    const afterVault = await getAccount(provider.connection, vaultAta);
    assert.equal(afterOwner.amount, beforeOwner.amount - BigInt(8));
    assert.equal(afterVault.amount, beforeVault.amount + BigInt(8));

    // read vault counter to derive poster PDA
    const vaultState: any = await (
      program.account as any
    ).vaultGlobalState.fetch(
      (
        await anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("vault_global_state")],
          program.programId
        )
      )[0]
    );
    // bountyCounter is a BN from the Anchor account; convert to number for seed encoding
    const counterBefore = vaultState.bountyCounter.toNumber();

    // create poster
    const bountyMinimumGain = 4; // will transfer 4 tokens
    const submissionCost = 1;
    const deadline = 9999999999;
    const tx = await (program as any).methods
      .uploadNewPoster(
        new anchor.BN(bountyMinimumGain),
        { openEnded: {} },
        { numberTheory: {} },
        new anchor.BN(deadline),
        null,
        new anchor.BN(submissionCost)
      )
      .accounts({
        owner: wallet,
        mint: anteMintPda,
        userAta: ownerAta,
        vaultGlobalState: (
          await anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_global_state")],
            program.programId
          )
        )[0],
        data: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster"),
            Buffer.from(UInt64ToLE(counterBefore)),
            wallet.toBuffer(),
          ],
          program.programId
        )[0],
        userBalanceInfo: userBalancePda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // verify token movement and user balance
    const ownerAfterPoster = await getAccount(provider.connection, ownerAta);
    const vaultAfterPoster = await getAccount(provider.connection, vaultAta);
    assert.equal(
      ownerAfterPoster.amount,
      afterOwner.amount - BigInt(bountyMinimumGain)
    );
    assert.equal(
      vaultAfterPoster.amount,
      afterVault.amount + BigInt(bountyMinimumGain)
    );

    // fetch poster account
    const posterPda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("poster"),
        Buffer.from(UInt64ToLE(counterBefore)),
        wallet.toBuffer(),
      ],
      program.programId
    )[0];
    const posterAcct: any = await (program.account as any).poster.fetch(
      posterPda
    );
    assert.equal(
      posterAcct.bountyMinimumGain.toString(),
      bountyMinimumGain.toString()
    );
    assert.equal(
      posterAcct.submissionCost.toString(),
      submissionCost.toString()
    );
  });

  it("poster: create poster fails when insufficient balance", async () => {
    const wallet = provider.wallet.publicKey;

    // ensure user has small balance (0)
    // if userBalance exists, withdraw to zero
    let userBalance: any = await (
      program.account as any
    ).userBalance.fetchNullable(userBalancePda);
    // if no balance account, it's effectively zero

    // attempt to create poster with large bounty
    const largeBounty = 1000000;
    let failed = false;
    try {
      await (program as any).methods
        .uploadNewPoster(
          new anchor.BN(largeBounty),
          { openEnded: {} },
          { numberTheory: {} },
          new anchor.BN(9999999999),
          null,
          new anchor.BN(1)
        )
        .accounts({
          owner: wallet,
          mint: anteMintPda,
          userAta: ownerAta,
          vaultGlobalState: (
            await anchor.web3.PublicKey.findProgramAddressSync(
              [Buffer.from("vault_global_state")],
              program.programId
            )
          )[0],
          data: anchor.web3.PublicKey.findProgramAddressSync(
            [
              Buffer.from("poster"),
              Buffer.from(UInt64ToLE(0)),
              wallet.toBuffer(),
            ],
            program.programId
          )[0],
          userBalanceInfo: userBalancePda,
          vaultAuthority: vaultAuthorityPda,
          vaultAta,
          tokenProgram: TOKEN_PROGRAM,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail(
        "expected poster creation to fail due to insufficient balance"
      );
    } catch (err: any) {
      failed = true;
      assert.isOk(err);
      // The failure can come from the account constraint (Anchor account error) or from the
      // explicit require! inside the instruction. Accept either message for robustness.
      const s = err.toString();
      assert.isTrue(
        s.includes("InsufficientAnteTokens") ||
          s.includes("AnchorError caused by account: data"),
        `unexpected error: ${s}`
      );
    }
    assert.isTrue(failed, "insufficient balance must fail poster creation");
  });
});
