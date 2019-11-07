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