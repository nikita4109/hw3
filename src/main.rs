use web3::{
    contract::{Contract, Options},
    ethabi::Token,
    transports::Http,
    types::{Address, BlockNumber, Bytes, TransactionParameters, H256, U256},
    types::{FilterBuilder, Log},
    Web3,
};

const POLYGON_MUMBAI_RPC_URL: &str = "https://rpc.ankr.com/polygon_mumbai";

struct StorageAPI {
    web3: Web3<Http>,
    private_key: String,
    contract: Contract<Http>,
    contract_address: Address,
    eoa: Address,
}

impl StorageAPI {
    fn new(private_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let transport = Http::new(POLYGON_MUMBAI_RPC_URL)?;
        let web3 = Web3::new(transport);

        let contract_address: Address = "0x7663333538ac92f1b71aFe6aa488d1f0EA709557".parse()?;
        let contract_abi = include_bytes!("../abi/storage.abi");
        let contract = Contract::from_json(web3.eth(), contract_address, contract_abi)?;

        let eoa: Address = "0xBff4622757Eb041e8ED34cf5A18902D94E586920".parse()?;

        Ok(Self {
            web3,
            private_key: private_key.into(),
            contract,
            contract_address,
            eoa,
        })
    }

    async fn send_transaction(
        &self,
        function_name: &str,
        tokens: &[Token],
    ) -> Result<H256, Box<dyn std::error::Error>> {
        let gas_price = self.web3.eth().gas_price().await?;
        let nonce = self.web3.eth().transaction_count(self.eoa, None).await?;

        let data = self
            .contract
            .abi()
            .function(function_name)?
            .encode_input(tokens)?;

        let tx = TransactionParameters {
            to: Some(self.contract_address),
            nonce: Some(nonce),
            gas_price: Some(gas_price),
            gas: U256::from(3_000_000),
            value: U256::from(0),
            data: Bytes(data),
            ..Default::default()
        };

        let signed_tx = self
            .web3
            .accounts()
            .sign_transaction(tx, &self.private_key.parse()?)
            .await?;
        let hash = self
            .web3
            .eth()
            .send_raw_transaction(signed_tx.raw_transaction)
            .await?;

        Ok(hash)
    }

    async fn add_item(&self, a: u64, b: &str, c: bool) -> Result<H256, Box<dyn std::error::Error>> {
        self.send_transaction(
            "addItem",
            &[
                Token::Uint(U256::from(a)),
                Token::String(b.to_owned()),
                Token::Bool(c),
            ],
        )
        .await
    }

    async fn remove_item(&self, a: u64) -> Result<H256, Box<dyn std::error::Error>> {
        self.send_transaction("removeItem", &[Token::Uint(U256::from(a))])
            .await
    }

    async fn query_events(
        &self,
        from_block: u64,
        to_block: u64,
        event_name: &str,
    ) -> Result<Vec<Log>, Box<dyn std::error::Error>> {
        let event_signature = self.contract.abi().event(event_name)?.signature();

        let filter = FilterBuilder::default()
            .from_block(BlockNumber::from(from_block))
            .to_block(BlockNumber::from(to_block))
            .address(vec![self.contract_address])
            .topics(Some(vec![event_signature]), None, None, None)
            .build();

        let logs = self.web3.eth().logs(filter).await?;
        Ok(logs)
    }
}

#[tokio::main]
async fn main() {
    let api = StorageAPI::new("INSERT PKEY HERE").expect("Failed to create API");

    // match api.add_item(1, "qwerty", true).await {
    //     Ok(hash) => println!("Transaction sent: {:?}", hash),
    //     Err(e) => eprintln!("Error sending transaction: {}", e),
    // }

    // match api.add_item(5, "abc", false).await {
    //     Ok(hash) => println!("Transaction sent: {:?}", hash),
    //     Err(e) => eprintln!("Error sending transaction: {}", e),
    // }

    // match api.remove_item(1).await {
    //     Ok(hash) => println!("Transaction sent: {:?}", hash),
    //     Err(e) => eprintln!("Error sending transaction: {}", e),
    // }

    match api.query_events(42862678, 42866046, "ItemAdded").await {
        Ok(logs) => {
            for log in logs {
                println!("{:?}", log);
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}
