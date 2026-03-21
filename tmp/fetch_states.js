const { ethers } = require("ethers");

async fn main() {
    const provider = new ethers.JsonRpcProvider("https://mainnet.base.org");
    const blockNumber = 43652157;

    const pools = [
        { name: "V3_005", address: "0xd0b53D9277642d899DF5C87A3966A349A798F224", type: "V3" },
        { name: "V3_03", address: "0x6c561b446416e1a00e8e93e221854d6ea4171372", type: "V3" },
        { name: "V2_Volatile", address: "0xcdac0d6c6c59727a65f871236188350531885c43", type: "V2" }
    ];

    for (const pool of pools) {
        console.log(`--- ${pool.name} ---`);
        if (pool.type === "V3") {
            const contract = new ethers.Contract(pool.address, ["function slot0() view returns (uint160, int24, uint16, uint16, uint16, uint8, bool)", "function liquidity() view returns (uint128)"], provider);
            const slot0 = await contract.slot0({ blockTag: blockNumber });
            const liquidity = await contract.liquidity({ blockTag: blockNumber });
            console.log(`sqrtPriceX96: ${slot0[0].toString()}`);
            console.log(`tick: ${slot0[1]}`);
            console.log(`liquidity: ${liquidity.toString()}`);
        } else {
            const contract = new ethers.Contract(pool.address, ["function getReserves() view returns (uint112, uint112, uint32)"], provider);
            const reserves = await contract.getReserves({ blockTag: blockNumber });
            console.log(`reserve0: ${reserves[0].toString()}`);
            console.log(`reserve1: ${reserves[1].toString()}`);
        }
    }
}

main();
