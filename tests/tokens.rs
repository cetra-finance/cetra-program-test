use cetra_program_test::{solana_program_test::*, *};
use solana_sdk::signer::Signer;

const USDT_AMOUNT: u64 = 10000000000;
const WSOL_AMOUNT: u64 = 1000000000;

#[tokio::test(flavor = "multi_thread")]
async fn success() {
    let rpc_accounts_loader = RpcAccountsLoader::default();
    let mut program_test_loader = ProgramTestLoader::default();

    program_test_loader.load().unwrap();
    let mut test_context = program_test_loader
        .start_with_context(Box::new(rpc_accounts_loader))
        .await;

    let wallet = test_context.context.payer.pubkey();

    test_context
        .create_ata(&wallet, &tokens::usdt::id(), USDT_AMOUNT)
        .await
        .expect("Unable to create USDT ata!");

    assert_eq!(
        test_context
            .get_ata_balance(&wallet, &tokens::usdt::id())
            .await
            .expect("Unable to get ata balance!"),
        USDT_AMOUNT
    );

    test_context
        .add_ata_tokens(&wallet, &tokens::usdt::id(), 1000)
        .await
        .expect("Unable to add ata tokens");

    assert_eq!(
        test_context
            .get_ata_balance(&wallet, &tokens::usdt::id())
            .await
            .expect("Unable to get ata balance!"),
        USDT_AMOUNT + 1000
    );

    test_context
        .create_ata(&wallet, &spl_token::native_mint::id(), WSOL_AMOUNT)
        .await
        .expect("Unable to create WSOL ata!");

    assert_eq!(
        test_context
            .get_ata_balance(&wallet, &spl_token::native_mint::id())
            .await
            .expect("Unable to get ata balance!"),
        WSOL_AMOUNT
    );

    test_context
        .add_ata_tokens(&wallet, &spl_token::native_mint::id(), 1000)
        .await
        .expect("Unable to add ata tokens");

    assert_eq!(
        test_context
            .get_ata_balance(&wallet, &spl_token::native_mint::id())
            .await
            .expect("Unable to get ata balance!"),
        WSOL_AMOUNT + 1000
    );
}
