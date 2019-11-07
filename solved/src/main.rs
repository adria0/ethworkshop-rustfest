use rustc_hex::FromHex;
use easycontract::{Account,EasyContract};
use web3::futures::Future;
use web3::types::*;

fn main() {

  let rpc_url = "https://goerli.infura.io/v3/"+ ... ;

  // do not let event loop goes out of scope!
  let (eloop, transport) = web3::transports::Http::new(rpc_url)
    .expect("cannot create web3 connector");
  eloop.into_remote();

  let web3 = web3::Web3::new(transport);

  // exercise 1 - read a block ---------------------------------------------------------------------------------

  let block = web3.eth().block(BlockId::Number(BlockNumber::Number(1574291)))
    .wait()
    .expect("cannot read block");

  println!("{:#?}", block);

  // exercise 2 - read a tx ---------------------------------------------------------------------------------
  
  let txraw  = "cb51297146cf222b7efa20ec0c2a098448ac68762f5e8d6c31554c0ad3d15499".from_hex::<Vec<u8>>().unwrap();
  let txhash = H256::from_slice(&txraw);
  let tx = web3.eth().transaction(TransactionId::Hash(txhash))
    .wait()
    .expect("connot read tx");

  println!("{:#?}", tx);

  // exercise 3 - yeah ---------------------------------------------------------------------------------

  let account1  = Account::from_secret_key("73b444918e17f54910e117e3d84a1efa4f3a2b2e994de3ca0348600f08c9fc8c");

  println!("address1={:x}",account1.address());

  let evmcode = (include_str!("Yeah.bin"))
    .from_hex::<Vec<u8>>()
    .expect("cannot parse evmcode");
  
  let abi_json = include_bytes!("Yeah.abi");

  let address = EasyContract::deploy(&web3, &account1, evmcode,U256::zero())
    .expect("cannot deploy contract");
  
  println!("Contract deployed at {:x}",address);

  let contract = EasyContract::from_json(&web3, address, abi_json)
    .expect("cannot assign to contract");
  
  let yeahs : U256 = contract.query("yeahs", (), None)
    .expect("cannot get yeahs");

  println!("yeahs = {}",yeahs);

  contract.call("yeah", (), &account1, U256::zero())
    .expect("cannot call yeah");
  
  let yeahs : U256 = contract.query("c", (), None)
    .expect("cannt get yeahs");

  println!("yeahs = {}",yeahs);

  // exercise 4 - economy ----------------------------------------------------------------------------------------
  // deploys the economy contract && transfers 10 tokens from account1 to account2
  
  let account2  = Account::from_secret_key("73b444918e17f54910e117e3d84a1efa4f3a2b2e994de3ca0348600f08c9fc8d");
  println!("address2={:x}",account2.address());

  let evmcode = (include_str!("Economy.bin"))
    .from_hex::<Vec<u8>>()
    .expect("cannot parse evmcode");
  
  let abi_json = include_bytes!("Economy.abi");

  let address = EasyContract::deploy(&web3, &account1, evmcode,U256::zero())
    .expect("cannot deploy contract");
  
  println!("Contract deployed at {:x}",address);
  
  let contract = EasyContract::from_json(&web3, address, abi_json)
    .expect("cannot assign to contract");
  
  contract.call("transfer", ( *account2.address(), U256::from(10) ), &account1, U256::zero() )
    .expect("cannot call contract");

  let balance1 : U256 = contract.query("balance", *account1.address(), None).expect("balance1");
  let balance2 : U256 = contract.query("balance", *account2.address(), None).expect("balance2");

  println!("balance1 = {}",balance1);
  println!("balance2 = {}",balance2);

}
