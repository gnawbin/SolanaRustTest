use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // solana_test::createTokenMint().await?;
    // solana_test::createTransferTokens().await?;
    // solana_test::testGetAccountInfo().await?;
    // solana_test::testGetInflationReward().await?;
    solana_test::testFeeForMessage().await?;
    Ok(())
}
