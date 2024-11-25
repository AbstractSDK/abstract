pub struct PythOracleTester {}

impl MockOracle for PythOracleTester {}

#[test]
fn test_price_query() -> anyhow::Result<()> {
    let dex_tester = setup_standard_pool()?;
    oracle_tester.test_price()?;
    Ok(())
}
