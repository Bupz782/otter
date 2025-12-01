#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asset {
    Eth,
    Dai,
    Usdc,
    Wbtc,
    Link,
    Sol,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Aave,
    Uniswap,
    Compound,
    Sushiswap,
    Balancer,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Swap {
        from_asset: Asset,
        to_asset: Asset,
        amount: u64,
        protocol: Protocol,
    },
    Stake {
        asset: Asset,
        amount: u64,
        protocol: Protocol,
    },
    Borrow {
        asset: Asset,
        amount: u64,
        protocol: Protocol,
    },
    Lend {
        asset: Asset,
        amount: u64,
        protocol: Protocol,
    },
}