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
pub enum DexType {
    Uniswap,
    Sushiswap,
    Balancer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LendingType {
    Aave,
    Compound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Dex(DexType),
    Lending(LendingType),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Swap {
        from_asset: Asset,
        to_asset: Asset,
        amount: u64,
        protocol: DexType,
    },
    Stake {
        asset: Asset,
        amount: u64,
        protocol: LendingType,
    },
    Borrow {
        asset: Asset,
        amount: u64,
        protocol: LendingType,
    },
    Lend {
        asset: Asset,
        amount: u64,
        protocol: LendingType,
    },
}