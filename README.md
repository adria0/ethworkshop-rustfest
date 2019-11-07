# 6th Ethereum Workshop (RustFest edition)

Slides https://github.com/adria0/slides/blob/master/ethereumdevbcn-workshop-rustfested.pdf

**THIS LIBRARY IS ONLY FOR EDUCATIONAL PURPOSES, DO NOT USE IN PRODUCTION**

## KATA1
- create a new rust project, add crates
    - `web3 = "0.8.0"`
    - `easycontract = { git = "https://github.com/adria0/ethworkshop-rustfest" }`

- initialize a new web3 with

```
  let (eloop, transport) = web3::transports::Http::new(rpc_url).expect("cannot connect to web3");
  eloop.into_remote();
  let web3 = web3::Web3::new(transport);
```

for the `rpc_url` use `https://goerli.infura.io/v3/<code>`

- read the contents of the block number `1574291` by using `web3.eth().block(...)` and dump its content the content to the screen (just `{:?}` )

- read it from etherscan  https://goerli.etherscan.io/block/1574291, check for the contents of the first transaction

- read the contents of the first block transaction programatically by using `web3.eth().transaction(...)`

## KATA2

- create random 32 byte random number in form of 64 hex chars string 
- initialize a new `easycontract::Account` using `from_secret_key` with the random string
- print the `.address` and claim some ethers in https://goerli-faucet.slock.it
- install the `solc` solidity compiler (https://solidity.readthedocs.io/en/v0.5.3/installing-solidity.html), install via `apt`, `snap` or `brew`
- create the file `Yeah.sol` with following content:

```
pragma solidity ^0.5.0;
contract Yeah {
  uint public yeahs;
  function yeah() public {
     yeahs++;
  } 
}
```
- compile the source with `solc -o . --bin --abi Yeah.sol`, it will create the `Yeah.bin` and `Yeah.abi`
  - check the file `Yeah.bin`, contains the compiled code
  - check the file `Yeah.abi`, contains the definition of the functions using json format  

- use `easycontract::Contract::deploy()` to create the smartcontract (use `U256::zero()` as value, and the unhexified contents of `Yeah.bin`) (TIP: `include_str!` and crate `rustc-hex`)

- use the address returned by the previous step and create an accessor by using ` easycontract::Contract::from_json(...)` 

- send a transaction (write on the blockchain) with`contract.call("yeah", (), &account, U256::zero())`

- call a function (read the blockchain) with `let yeahs : U256 = contract.query("yeahs", (), None).expect("call yeah");` 

- check the results on ehterscan

## KATA3

- use the Economy, or another - or - write your own
- deploy it and interact with it (in economy you can transfer 10 tokens from one account to another)


```
pragma solidity ^0.5.0;
contract Economy {
  mapping(address=>uint) public balance;
  constructor() public {
    balance[msg.sender] = 1000000;
  }
  function transfer(address to, uint amount) public {
     require(balance[msg.sender]>=amount);
     balance[msg.sender]-=amount;
     balance[to]+=amount;
  } 
}
```

```
pragma solidity ^0.5.0;
contract Property {
  mapping(bytes32=>address) public owner;
  function transfer(bytes32 assetid, address newowner) public {
     if (msg.sender==newowner && owner[assetid]==address(0)) {
        owner[assetid]=newowner;
     } else {
        require(owner[assetid]==msg.sender);
        owner[assetid] = newowner;
     }
  }
}
```

```
pragma solidity ^0.5.0;
contract Identity {
  struct identity {
      string name;
      mapping (address=>bool) approvals;
  }
  mapping(address=>identity) public ids;
  function claim(string calldata name) external {
      require(bytes(ids[msg.sender].name).length==0);
      ids[msg.sender].name=name;
  }
  function trust(address who) external {
      ids[who].approvals[msg.sender]=true;
  }
}
```

```
pragma solidity ^0.5.0;
contract Democracy {
  struct poll {
    mapping(uint256=>uint) result;
    mapping(address=>bool) voted;
  }
  mapping(uint256=>poll) polls;
  function vote(uint pollno, uint option) public {
    require(polls[pollno].voted[msg.sender]==false);
    polls[pollno].voted[msg.sender]=true;
    polls[pollno].result[option]+=1;
  }
  function votes(uint pollno, uint option) public view returns (uint) {
      return polls[pollno].result[option];
  } 
}
```
