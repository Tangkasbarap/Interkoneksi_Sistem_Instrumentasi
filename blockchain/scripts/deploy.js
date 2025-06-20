const hre = require("hardhat");
const fs = require("fs");

async function main() {
  const marketplace = await hre.ethers.deployContract("DataMarketplace");
  await marketplace.waitForDeployment();
  const contractAddress = marketplace.target;
  console.log(`DataMarketplace di-deploy ke: ${contractAddress}`);

  const contractData = {
    address: contractAddress,
    abi: JSON.parse(marketplace.interface.formatJson())
  };

  fs.writeFileSync(__dirname + "/../../frontend/public/deployedAddress.json", JSON.stringify(contractData, null, 2));
  console.log("✅ Alamat & ABI disimpan di deployedAddress.json");
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
