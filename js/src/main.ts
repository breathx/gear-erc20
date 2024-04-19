import { GearApi, decodeAddress } from '@gear-js/api';
import { Keyring } from '@polkadot/api';
import { readFileSync } from 'fs';
import { IKeyringPair } from '@polkadot/types/types';
import assert from 'assert/strict';
import { ERC20 } from './lib.js';

const upload = async (api: GearApi, account: IKeyringPair) => {
  const code = readFileSync('../erc20_wasm.opt.wasm');

  const grc = new ERC20(api);

  const transaction = await grc.newCtorFromCode(code, 'VARA_TOKEN', 'TOK', 5).withAccount(account).calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  console.log(`\nInit message added to block ${blockHash}. Message id: ${msgId}`);

  const result = await response();

  console.log('Init message executed successfully.');

  return grc;
};

const mint = async (grc: ERC20, account: IKeyringPair) => {
  const transaction = await grc
    .setBalance(BigInt(100 * 1e5))
    .withAccount(account)
    .calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  console.log(`\nMint message added to block ${blockHash}. Message id: ${msgId}`);

  const result = await response();

  console.log('Mint message executed successfully.');
};

const transfer = async (grc: ERC20, account: IKeyringPair, to: string) => {
  const transaction = await grc
    .transfer(decodeAddress(to), BigInt(10 * 1e5))
    .withAccount(account)
    .calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  console.log(`\nTransfer message added to block ${blockHash}. Message id: ${msgId}`);

  const result = await response();

  console.log('Transfer message executed successfully.');
};

const approve = async (grc: ERC20, account: IKeyringPair, spender: string) => {
  const transaction = await grc
    .approve(decodeAddress(spender), BigInt(50 * 1e5))
    .withAccount(account)
    .calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  console.log(`\nApprove message added to block ${blockHash}. Message id: ${msgId}`);

  const result = await response();

  console.log('Approve message executed successfully.');
};

const transferFrom = async (grc: ERC20, account: IKeyringPair, from: string, to: string) => {
  const transaction = await grc
    .fromTransfer(decodeAddress(from), decodeAddress(to), BigInt(20 * 1e5))
    .withAccount(account)
    .calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  console.log(`\nTransferFrom message added to block ${blockHash}. Message id: ${msgId}`);

  const result = await response();

  console.log('TransferFrom message executed successfully.');
};

const main = async () => {
  const providerAddress = process.argv.length > 2 ? process.argv[2] : 'ws://127.0.0.1:9944';

  if (!providerAddress.startsWith('ws')) {
    throw new Error('Invalid provider address');
  }

  const api = await GearApi.create({ providerAddress });
  const keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });

  const alice = keyring.addFromUri('//Alice');
  const aliceAddress = decodeAddress(alice.address);
  const bob = keyring.addFromUri('//Bob');
  const bobAddress = decodeAddress(bob.address);
  const charlie = keyring.addFromUri('//Charlie');
  const charlieAddress = decodeAddress(charlie.address);

  // Upload ERC20 contract
  const grc = await upload(api, alice);

  // Mint 100 tokens to Alice
  await mint(grc, alice);

  // Check Alice balance
  let aliceBalance = BigInt(await grc.balanceOf(aliceAddress, aliceAddress));
  console.log('\nAlice balance: ', aliceBalance);
  assert.strictEqual(aliceBalance, BigInt(100 * 1e5), 'Alice balance should be 100');

  // Transfer 10 tokens from Alice to Bob
  await transfer(grc, alice, bob.address);

  aliceBalance = BigInt(await grc.balanceOf(aliceAddress, aliceAddress));
  assert.strictEqual(aliceBalance, BigInt(90 * 1e5), 'Alice balance should be 90');
  console.log('\nAlice balance: ', aliceBalance);

  let bobBalance = BigInt(await grc.balanceOf(bobAddress, bobAddress));
  assert.strictEqual(bobBalance, BigInt(10 * 1e5), 'Bob balance should be 10');
  console.log('Bob balance: ', bobBalance);

  // Approve Bob to transfer 50 tokens from Alice
  await approve(grc, alice, bob.address);

  // Transfer 20 tokens from Alice to Charlie using Bob
  await transferFrom(grc, bob, alice.address, charlie.address);

  aliceBalance = BigInt(await grc.balanceOf(aliceAddress, aliceAddress));
  assert.strictEqual(aliceBalance, BigInt(70 * 1e5), 'Alice balance should be 70');
  console.log('\nAlice balance: ', aliceBalance);
  bobBalance = BigInt(await grc.balanceOf(bobAddress, bobAddress));
  assert.strictEqual(bobBalance, BigInt(10 * 1e5), 'Bob balance should be 10');
  console.log('Bob balance: ', bobBalance);
  let charlieBalance = BigInt(await grc.balanceOf(charlieAddress, charlieAddress));
  assert.strictEqual(charlieBalance, BigInt(20 * 1e5), 'Charlie balance should be 20');
  console.log('Charlie balance: ', charlieBalance, '\n');
};

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.log(error);
    process.exit(1);
  });
