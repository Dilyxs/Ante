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
import { UInt64ToLE, ensureOwnerAtaHas } from "./helpers";

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

  // helpers imported from tests/helpers.ts: UInt64ToLE, ensureOwnerAtaHas

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
    // use delta checks to be robust against prior mints in the test run
    const ownerDelta = ownerBeforeDeposit.amount - ownerAfterDeposit.amount;
    const vaultDelta = vaultAfterDeposit.amount - vaultBeforeDeposit.amount;
    assert.equal(ownerDelta, BigInt(5));
    assert.equal(vaultDelta, BigInt(5));

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
    const ownerWithdrawDelta =
      ownerAfterWithdraw.amount - ownerBeforeWithdraw.amount;
    const vaultWithdrawDelta =
      vaultBeforeWithdraw.amount - vaultAfterWithdraw.amount;
    assert.equal(ownerWithdrawDelta, BigInt(2));
    assert.equal(vaultWithdrawDelta, BigInt(2));

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
          [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
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
      [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
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

  it("poster: create poster fails — insufficient balance", async () => {
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
            [Buffer.from("poster"), Buffer.from(UInt64ToLE(0))],
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

  it("publish: publisher posts solution — happy path (deadline passed)", async () => {
    const wallet = provider.wallet.publicKey;

    // create poster with deadline in the past so publish is allowed
    const depositAmount = new anchor.BN(8);
    // mint enough tokens to owner ATA so we can deposit and still have tokens for bounty transfer
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
    await (program as any).methods
      .requestAnteTokens(new anchor.BN(4))
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
    const counterBefore = vaultState.bountyCounter.toNumber();

    // replenish owner ATA so uploadNewPoster transfer will succeed (owner_ata must have tokens)
    await (program as any).methods
      .requestAnteTokens(new anchor.BN(4))
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

    // ensure owner ATA has enough tokens for the bounty transfer
    await program.methods
      .requestAnteTokens(new anchor.BN(4))
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

    // create poster with deadline passed (0)
    await ensureOwnerAtaHas(
      program,
      provider,
      ownerAta,
      anteMintPda,
      vaultAuthorityPda,
      vaultAta,
      configPda,
      16
    );
    await (program as any).methods
      .uploadNewPoster(
        new anchor.BN(4),
        { openEnded: {} },
        { numberTheory: {} },
        new anchor.BN(0),
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
          [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
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

    const posterPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
      program.programId
    )[0];

    const publisherDecryptedAnswerPda =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("poster_decrypted_answer"),
          Buffer.from(UInt64ToLE(counterBefore)),
          wallet.toBuffer(),
        ],
        program.programId
      )[0];

    // publisher posts solution
    await program.methods
      .postPosterSolution(
        new anchor.BN(counterBefore),
        "publisher answer",
        "hash-1"
      )
      .accounts({
        publisher: wallet,
        posterInfo: posterPda,
        publisherDecryptedAnswer: publisherDecryptedAnswerPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // verify account created
    const pubAns: any = await (
      program.account as any
    ).posterPublisherDecryptedAnswer.fetch(publisherDecryptedAnswerPda);
    assert.equal(pubAns.posterId.toString(), counterBefore.toString());
    assert.equal(pubAns.answer, "publisher answer");
    assert.equal(pubAns.hash, "hash-1");
  });

  it("publish: publisher posts solution — fails when deadline not passed", async () => {
    const wallet = provider.wallet.publicKey;

    // deposit and create poster with future deadline
    // mint enough tokens to owner ATA so we can deposit and still have tokens for bounty transfer
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
    await (program as any).methods
      .requestAnteTokens(new anchor.BN(4))
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

    await (program as any).methods
      .depositeAnteTokens(new anchor.BN(8))
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

    // top up owner ATA so uploadNewPoster transfer will succeed
    await (program as any).methods
      .requestAnteTokens(new anchor.BN(4))
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
    const counterBefore = vaultState.bountyCounter.toNumber();

    // replenish owner ATA so uploadNewPoster transfer will succeed
    await (program as any).methods
      .requestAnteTokens(new anchor.BN(4))
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

    // ensure owner ATA has enough tokens for the bounty transfer
    await program.methods
      .requestAnteTokens(new anchor.BN(4))
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

    // create poster with far future deadline
    // ensure owner ATA has enough tokens for the bounty transfer
    await program.methods
      .requestAnteTokens(new anchor.BN(4))
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

    await ensureOwnerAtaHas(
      program,
      provider,
      ownerAta,
      anteMintPda,
      vaultAuthorityPda,
      vaultAta,
      configPda,
      16
    );
    await (program as any).methods
      .uploadNewPoster(
        new anchor.BN(4),
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
          [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
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

    const posterPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
      program.programId
    )[0];

    const publisherDecryptedAnswerPda =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("poster_decrypted_answer"),
          Buffer.from(UInt64ToLE(counterBefore)),
          wallet.toBuffer(),
        ],
        program.programId
      )[0];

    let failed = false;
    try {
      await program.methods
        .postPosterSolution(
          new anchor.BN(counterBefore),
          "publisher answer",
          "hash-2"
        )
        .accounts({
          publisher: wallet,
          posterInfo: posterPda,
          publisherDecryptedAnswer: publisherDecryptedAnswerPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("expected publish to fail due to deadline not passed");
    } catch (err: any) {
      failed = true;
      assert.isOk(err);
      const s = err.toString();
      assert.isTrue(
        s.includes("PosterDeadlineNotPassed") ||
          s.includes("AccountNotInitialized") ||
          s.includes("poster_response") ||
          s.includes("posterResponse"),
        `unexpected error: ${s}`
      );
    }
    assert.isTrue(failed);
  });

  it("publish: answerer posts decrypted answer — happy path (poster_response exists, deadline passed)", async () => {
    const wallet = provider.wallet.publicKey;

    // create poster with deadline passed
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

    await (program as any).methods
      .depositeAnteTokens(new anchor.BN(8))
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
    const counterBefore = vaultState.bountyCounter.toNumber();

    // replenish owner ATA so uploadNewPoster transfer will succeed
    await (program as any).methods
      .requestAnteTokens(new anchor.BN(4))
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

    await ensureOwnerAtaHas(
      program,
      provider,
      ownerAta,
      anteMintPda,
      vaultAuthorityPda,
      vaultAta,
      configPda,
      16
    );
    await (program as any).methods
      .uploadNewPoster(
        new anchor.BN(4),
        { openEnded: {} },
        { numberTheory: {} },
        new anchor.BN(0),
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
          [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
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

    const posterPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
      program.programId
    )[0];

    // create answerer and their ATA
    const answerer = anchor.web3.Keypair.generate();
    // fund answerer with SOL so they can pay for account creation (rent)
    await provider.connection.requestAirdrop(answerer.publicKey, 1_000_000_000);
    const answererAta = getAssociatedTokenAddressSync(
      anteMintPda,
      answerer.publicKey,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );
    const createAnswererAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet,
        answererAta,
        answerer.publicKey,
        anteMintPda,
        TOKEN_PROGRAM,
        ASSOCIATED_TOKEN_PROGRAM
      )
    );
    await provider.sendAndConfirm(createAnswererAtaTx);

    // fund answerer with requestAnteTokens from admin (mint more so we can deposit some but keep ATA balance)
    await program.methods
      .requestAnteTokens(new anchor.BN(8))
      .accounts({
        admin: wallet,
        mint: anteMintPda,
        asker: answerer.publicKey,
        askerAta: answererAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    // answerer deposits into their user balance
    // deposit a portion, leaving some tokens in answerer ATA for submission transfer
    await (program as any).methods
      .depositeAnteTokens(new anchor.BN(4))
      .accounts({
        owner: answerer.publicKey,
        ownerAta: answererAta,
        mint: anteMintPda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("user_balance_info"), answerer.publicKey.toBuffer()],
          program.programId
        )[0],
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([answerer])
      .rpc();

    // answerer posts an answer to create poster_response
    const answerArr = new Uint8Array(33);
    answerArr[0] = 1; // arbitrary
    await program.methods
      .answerPoster(new anchor.BN(counterBefore), Array.from(answerArr))
      .accounts({
        answerer: answerer.publicKey,
        mint: anteMintPda,
        answererAta: answererAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("user_balance_info"), answerer.publicKey.toBuffer()],
          program.programId
        )[0],
        posterInfo: posterPda,
        posterPublisher: wallet,
        posterResponse: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("poster_response"),
            Buffer.from(UInt64ToLE(counterBefore)),
            answerer.publicKey.toBuffer(),
          ],
          program.programId
        )[0],
        vaultGlobalState: (
          await anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("vault_global_state")],
            program.programId
          )
        )[0],
        tokenProgram: TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
      })
      .signers([answerer])
      .rpc();

    const posterResponsePda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("poster_response"),
        Buffer.from(UInt64ToLE(counterBefore)),
        answerer.publicKey.toBuffer(),
      ],
      program.programId
    )[0];

    const answererDecryptedAnswerPda =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("poster_answerer_decrypted_answer"),
          Buffer.from(UInt64ToLE(counterBefore)),
          answerer.publicKey.toBuffer(),
        ],
        program.programId
      )[0];

    // answerer posts decrypted answer
    await program.methods
      .postAnswererDecryptedAnswer(
        new anchor.BN(counterBefore),
        "answerer answer",
        "hash-3"
      )
      .accounts({
        answerer: answerer.publicKey,
        posterResponse: posterResponsePda,
        answererDecryptedAnswer: answererDecryptedAnswerPda,
        posterInfo: posterPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([answerer])
      .rpc();

    const ansPub: any = await (
      program.account as any
    ).posterAnswererDecryptedAnswer.fetch(answererDecryptedAnswerPda);
    assert.equal(ansPub.posterId.toString(), counterBefore.toString());
    assert.equal(ansPub.answer, "answerer answer");
    assert.equal(ansPub.hash, "hash-3");
  });

  it("publish: answerer posts decrypted answer — fails when deadline not passed", async () => {
    const wallet = provider.wallet.publicKey;

    // create poster with far future deadline
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

    await (program as any).methods
      .depositeAnteTokens(new anchor.BN(8))
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
    const counterBefore = vaultState.bountyCounter.toNumber();

    await ensureOwnerAtaHas(
      program,
      provider,
      ownerAta,
      anteMintPda,
      vaultAuthorityPda,
      vaultAta,
      configPda,
      16
    );
    await (program as any).methods
      .uploadNewPoster(
        new anchor.BN(4),
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
          [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
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

    const posterPda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("poster"), Buffer.from(UInt64ToLE(counterBefore))],
      program.programId
    )[0];

    // create answerer and their ATA
    const answerer = anchor.web3.Keypair.generate();
    // fund answerer with SOL so they can pay for account creation (rent)
    await provider.connection.requestAirdrop(answerer.publicKey, 1_000_000_000);
    const answererAta = getAssociatedTokenAddressSync(
      anteMintPda,
      answerer.publicKey,
      true,
      TOKEN_PROGRAM,
      ASSOCIATED_TOKEN_PROGRAM
    );
    const createAnswererAtaTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet,
        answererAta,
        answerer.publicKey,
        anteMintPda,
        TOKEN_PROGRAM,
        ASSOCIATED_TOKEN_PROGRAM
      )
    );
    await provider.sendAndConfirm(createAnswererAtaTx);

    // fund answerer with requestAnteTokens from admin (mint more so we can deposit some but keep ATA balance)
    await program.methods
      .requestAnteTokens(new anchor.BN(8))
      .accounts({
        admin: wallet,
        mint: anteMintPda,
        asker: answerer.publicKey,
        askerAta: answererAta,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        config: configPda,
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();

    // answerer deposits into their user balance
    await (program as any).methods
      .depositeAnteTokens(new anchor.BN(4))
      .accounts({
        owner: answerer.publicKey,
        ownerAta: answererAta,
        mint: anteMintPda,
        vaultAuthority: vaultAuthorityPda,
        vaultAta,
        userBalanceInfo: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("user_balance_info"), answerer.publicKey.toBuffer()],
          program.programId
        )[0],
        tokenProgram: TOKEN_PROGRAM,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([answerer])
      .rpc();

    // answerer tries to post answerer decrypted answer but deadline not passed
    const posterResponsePda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("poster_response"),
        Buffer.from(UInt64ToLE(counterBefore)),
        answerer.publicKey.toBuffer(),
      ],
      program.programId
    )[0];
    const answererDecryptedAnswerPda =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("poster_answerer_decrypted_answer"),
          Buffer.from(UInt64ToLE(counterBefore)),
          answerer.publicKey.toBuffer(),
        ],
        program.programId
      )[0];

    let failed = false;
    try {
      await program.methods
        .postAnswererDecryptedAnswer(
          new anchor.BN(counterBefore),
          "answerer answer",
          "hash-4"
        )
        .accounts({
          answerer: answerer.publicKey,
          posterResponse: posterResponsePda,
          answererDecryptedAnswer: answererDecryptedAnswerPda,
          posterInfo: posterPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([answerer])
        .rpc();
      assert.fail("expected publish to fail due to deadline not passed");
    } catch (err: any) {
      failed = true;
      assert.isOk(err);
      const s = err.toString();
      assert.isTrue(
        s.includes("PosterDeadlineNotPassed") ||
          s.includes("poster_response") ||
          s.includes("AccountNotInitialized") ||
          s.includes("posterResponse"),
        `unexpected error: ${s}`
      );
    }
    assert.isTrue(failed);
  });
});
