use cetra_program_test::{solana_program_test::*, *};
use solana_program::program_pack::Pack;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use std::str::FromStr;

#[tokio::test(flavor = "multi_thread")]
async fn success() {
    let rpc_accounts_loader = RpcAccountsLoader::default();
    let mut program_test_loader = ProgramTestLoader::default();

    program_test_loader.load().unwrap();
    let mut test_context = program_test_loader
        .start_with_context(Box::new(rpc_accounts_loader))
        .await;

    // SOL reserve
    let reserve_pubkey = Pubkey::from_str("8PbodeaosQP19SjYFx855UMqWxH2HynZLdBXmsrbac36").unwrap();

    // Reserve liquidity oracle
    let pyth_price_pubkey =
        Pubkey::from_str("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG").unwrap();

    let mut pyth_price_account = test_context
        .get_account(&pyth_price_pubkey)
        .await
        .unwrap()
        .unwrap();

    let pyth_price: &mut solend_program::pyth::Price =
        solend_program::pyth::load_mut(&mut pyth_price_account.data).unwrap();

    // Also we can patch `Clock` sysvar
    test_context
        .context
        .warp_to_slot(pyth_price.valid_slot)
        .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[solend_program::instruction::refresh_reserve(
            solend_program::id(),
            reserve_pubkey,
            pyth_price_pubkey,
            Pubkey::from_str("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR").unwrap(),
        )],
        Some(&test_context.context.payer.pubkey()),
        &[&test_context.context.payer],
        test_context.context.last_blockhash,
    );

    test_context.process_transaction(tx).await.unwrap();

    let _reserve = solend_program::state::Reserve::unpack(
        &test_context
            .context
            .banks_client
            .get_account(reserve_pubkey)
            .await
            .unwrap()
            .unwrap()
            .data,
    )
    .unwrap();
}
