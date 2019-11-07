use std::str::FromStr;
use std::{thread, time};

use rustc_hex::FromHex;
use tiny_keccak;

use serde::Serialize;

use secp256k1::{Secp256k1, SecretKey, PublicKey};

use ethabi;
use web3::futures::Future;
use web3::types::*;
use web3::Web3;
use web3::Transport;
use web3::contract::tokens::{Tokenize,Detokenize};
use web3::contract::QueryResult;
use ethereum_tx_sign::RawTransaction;

pub struct Account {
  address : Address,
  private : H256,
}

impl Account {
  pub fn from_secret_key(secret_key_str : &str) -> Self {
    let secp = Secp256k1::new();

    let secret_key = SecretKey::from_str(secret_key_str).unwrap();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

		let mut keccak = tiny_keccak::Keccak::new_keccak256();
		let mut hash = [0u8; 32];
		keccak.update(&public_key.serialize_uncompressed()[1..65]);
		keccak.finalize(&mut hash);

    let address = Address::from_slice(&hash[12..]);
    let private = H256::from_slice(&secret_key_str.from_hex::<Vec<u8>>().unwrap());
    Account { address, private }
  }
  pub fn address(&self) -> &Address {
      &self.address
  }
}

/// Call contract request (eth_call / eth_estimateGas)
#[derive(Clone, Debug, PartialEq, Serialize)]
struct EstimateGasRequest<'a> {
    /// Sender address (None for arbitrary address)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// To address
    pub to: Option<Address>,
    /// Supplied gas (None for sensible default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,
    /// Gas price (None for sensible default)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<U256>,
    /// Transfered valueQueryResult (None for no transfer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    /// Data (None for empty data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<&'a Bytes>,
}

#[derive(Debug)]
pub enum EasyContractError {
  Contract(web3::contract::Error),
  Abi(ethabi::Error),
  Web3(web3::error::Error),
  Failed(String)
}

impl From<web3::error::Error> for EasyContractError {
  fn from(err : web3::error::Error) -> Self {
    EasyContractError::Web3(err)
  }
}

impl From<web3::contract::Error> for EasyContractError {
  fn from(err : web3::contract::Error) -> Self {
    EasyContractError::Contract(err)
  }
}

impl From<ethabi::Error> for EasyContractError {
  fn from(err : ethabi::Error) -> Self {
    EasyContractError::Abi(err)
  }
}

/// Ethereum Contract Interface
#[derive(Debug, Clone)]
pub struct EasyContract<'a, T: Transport> {
    address: Address,
    web3: &'a Web3<T>,
    abi: ethabi::Contract,
}

impl<'a, T: Transport> EasyContract<'a, T> {
    /// Creates new Contract Interface given blockchain address and ABI
    pub fn new(web3: &'a Web3<T>, address: Address, abi: ethabi::Contract) -> Self {
        EasyContract { address, web3, abi  }
    }

    /// Creates new Contract Interface given blockchain address and JSON containing ABI
    pub fn from_json(web3: &'a Web3<T>, address: Address, json: &[u8]) -> Result<Self, ethabi::Error> {
        let abi = ethabi::Contract::load(json)?;
        Ok(Self::new(web3, address, abi))
    }

    /// Delpoy a contract
    pub fn deploy(web3: &'a Web3<T>, account: &Account, evmcode: Vec<u8>, value: U256) -> Result<Address,EasyContractError> {
      let tx_recipt = Self::send_transaction(web3, &account, None, value, evmcode)?;
      Ok(tx_recipt.contract_address.unwrap())
    }

    /// Returns contract address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns abi
    pub fn abi(&self) -> &ethabi::Contract {
        &self.abi
    }

    /// Call constant function
    pub fn query<R, A, P>(
        &self,
        func: &str,
        params: P,
        from: A,
    ) -> std::result::Result<R, web3::contract::Error>
    where
        R: Detokenize,
        A: Into<Option<Address>>,
        P: Tokenize,
    {
        self.abi
            .function(func)
            .and_then(|function| {
                function
                    .encode_input(&params.into_tokens())
                    .map(|call| (call, function))
            })
            .map(|(call, function)| {
                let result = self.web3.eth().call(
                    CallRequest {
                        from: from.into(),
                        to: self.address,
                        gas: None,
                        gas_price: None,
                        value : None,
                        data: Some(Bytes(call)),
                    },
                    None,
                );
                QueryResult::new(result, function.clone())
            })
            .unwrap_or_else(Into::into)
            .wait()
    }

    /// Call function
    pub fn call<P>(
        &self,
        func: &str,
        params: P,
        from: &Account,
        value: U256,
    ) -> Result<TransactionReceipt,EasyContractError>
    where
        P: Tokenize,
    {
        let function = self.abi.function(func)?;
        let data = function.encode_input(&params.into_tokens())?;
        Self::send_transaction(&self.web3,from, Some(self.address), value, data)
    }

    /// Call perform transation and wait 
  fn send_transaction<TT : Transport>(
    web3: &Web3<TT>,
    from: &Account,
    to: Option<Address>,
    value: U256,
    data: Vec<u8>
  ) -> Result<TransactionReceipt,EasyContractError> {

    let nonce = web3.eth().transaction_count(from.address,None);
    let gas_price = web3.eth().gas_price();
    let chain_id = web3.net().version();
    let data = Bytes(data);

    let req = web3::helpers::serialize(&EstimateGasRequest {
      from: Some(from.address),
      to,
      gas: None,
      gas_price : None,
      value: Some(value),
      data: Some(&data),
    });

    let gas = web3::helpers::CallFuture::new(
      web3.transport().execute("eth_estimateGas", vec![req])
    );

    let nonce = nonce.wait()?;
    let gas_price = gas_price.wait()?;
    let chain_id = chain_id.wait()?.parse::<u8>().unwrap();
    let gas = gas.wait()?;
  
    let tx = RawTransaction {
        nonce,
        to,
        value,
        gas_price,
        gas,
        data : data.0,
    };

    let tx_signed = tx.sign(&from.private,&chain_id);
    let tx_hash = web3.eth().send_raw_transaction(Bytes(tx_signed)).wait()?;

    let mut receipt = None;
    while receipt.is_none() {
      println!("wating for receipt of tx {:x}",tx_hash);
      thread::sleep(time::Duration::from_secs(4));
      receipt = web3.eth().transaction_receipt(tx_hash).wait()?;
    }
    let receipt = receipt.unwrap();

    if receipt.status.unwrap().as_u64() == 1 {
      Ok(receipt)
    } else {
      Err(EasyContractError::Failed("transaction failed".to_string()))
    }

  }
}
